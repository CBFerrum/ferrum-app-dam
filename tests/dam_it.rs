//! A brand's library (FERRUM-ECOSYSTEM-012) against a real PostgreSQL
//! (`FERRUM_DAM_TEST_DATABASE_URL`, default `postgres://ferrum:ferrum@localhost:5432/ferrum_dam_test`;
//! reset first), with Drive installed: files uploaded to Drive made assets whose type, size and
//! digest come from the file; licences naming their licensor and days; approval by a Brand Manager
//! of the file's version then; a new version sending the asset back for review; usages only of the
//! approved version within its days, editorial-only assets not used to sell; expiry; renditions
//! and what a photo's file says of it, made again for a new version and seen by whoever sees the
//! asset; and who sees and does what.

use base64::Engine;
use chrono::{Duration, Utc};
use ferrum_kernel::secret::SecretKey;
use ferrum_kernel::tenant::NewTenant;
use ferrum_kernel::{Ctx, Db, DbConfig, Doc, Error, HookRegistry, Kernel, KernelSettings, ListQuery};
use ferrum_meta::Catalog;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

fn obj(v: Value) -> Doc {
    v.as_object().cloned().unwrap()
}

fn uid(v: &Value) -> Uuid {
    v.as_str().unwrap_or_else(|| panic!("not an id: {v}")).parse().unwrap()
}

/// An in-memory object store standing in for S3.
#[derive(Default)]
struct MemFiles(std::sync::Mutex<HashMap<String, Vec<u8>>>);

#[async_trait::async_trait]
impl ferrum_kernel::files::FileStore for MemFiles {
    async fn put(&self, key: &str, data: &[u8]) -> ferrum_kernel::Result<()> {
        self.0.lock().unwrap().insert(key.to_string(), data.to_vec());
        Ok(())
    }
    async fn get(&self, key: &str) -> ferrum_kernel::Result<Vec<u8>> {
        self.0.lock().unwrap().get(key).cloned().ok_or_else(|| Error::NotFound("object".into()))
    }
    async fn delete(&self, key: &str) -> ferrum_kernel::Result<()> {
        self.0.lock().unwrap().remove(key);
        Ok(())
    }
}

fn png(fill: u8) -> Vec<u8> {
    let mut v = b"\x89PNG\r\n\x1a\n".to_vec();
    v.resize(600, fill);
    v
}

/// A JPEG photo `w` by `h` with the EXIF a camera writes: when it was taken, with what, by whom,
/// under whose copyright, held on its side, and where.
fn photo(w: u32, h: u32, shade: u8) -> Vec<u8> {
    use exif::{Field, In, Rational, Tag, Value as Exif};
    let ascii = |tag: Tag, text: &str| Field { tag, ifd_num: In::PRIMARY, value: Exif::Ascii(vec![text.as_bytes().to_vec()]) };
    let whole = |n: u32| Rational { num: n, denom: 1 };
    let fields = [
        ascii(Tag::Make, "Canon"),
        ascii(Tag::Model, "Canon EOS R6"),
        ascii(Tag::Artist, "Jo Doe"),
        ascii(Tag::Copyright, "Brand Co"),
        ascii(Tag::DateTimeOriginal, "2026:05:01 10:30:00"),
        Field { tag: Tag::Orientation, ifd_num: In::PRIMARY, value: Exif::Short(vec![6]) },
        Field { tag: Tag::GPSLatitude, ifd_num: In::PRIMARY, value: Exif::Rational(vec![whole(26), whole(9), whole(0)]) },
    ];
    let mut writer = exif::experimental::Writer::new();
    for f in &fields {
        writer.push_field(f);
    }
    let mut tiff = std::io::Cursor::new(Vec::new());
    writer.write(&mut tiff, false).expect("EXIF");
    let tiff = tiff.into_inner();
    let mut jpeg = Vec::new();
    image::DynamicImage::ImageRgb8(image::RgbImage::from_fn(w, h, |x, y| image::Rgb([(x % 256) as u8, (y % 256) as u8, shade])))
        .write_with_encoder(image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, 80))
        .unwrap();
    // The EXIF goes in an APP1 segment right after the JPEG's start.
    let mut out = vec![0xFF, 0xD8, 0xFF, 0xE1];
    out.extend_from_slice(&u16::try_from(2 + 6 + tiff.len()).unwrap().to_be_bytes());
    out.extend_from_slice(b"Exif\0\0");
    out.extend_from_slice(&tiff);
    out.extend_from_slice(&jpeg[2..]);
    out
}

/// A stored file's id from its address.
fn file_id(url: &Value) -> Uuid {
    url.as_str().and_then(|u| u.strip_prefix("/files/")).unwrap_or_else(|| panic!("not a stored file: {url}")).parse().unwrap()
}

/// An image's size in pixels.
fn pixels(bytes: &[u8]) -> (u32, u32) {
    image::ImageReader::new(std::io::Cursor::new(bytes)).with_guessed_format().unwrap().into_dimensions().unwrap()
}

struct T {
    k: Arc<Kernel>,
    tenant: Uuid,
    admin: Ctx,
}

impl T {
    async fn insert(&self, ctx: &Ctx, entity: &str, v: Value) -> Doc {
        self.k.insert(ctx, entity, obj(v)).await.unwrap_or_else(|e| panic!("insert {entity}: {e:?}"))
    }
    async fn get(&self, entity: &str, id: Uuid) -> Doc {
        self.k.get(&self.admin, entity, id, false).await.unwrap_or_else(|e| panic!("get {entity}: {e:?}"))
    }
    async fn user(&self, email: &str, roles: &[&str]) -> Ctx {
        let roles: Vec<String> = roles.iter().map(|r| r.to_string()).collect();
        let id = self.k.create_user(&self.admin, email, email, Some("Correct-Horse-9"), &roles, false).await.expect("user");
        let s = self.k.login("brand", email, "Correct-Horse-9").await.expect("login");
        self.k.load_ctx(self.tenant, id, Some(s.token_version), "it".into()).await.expect("ctx")
    }
    async fn act(&self, ctx: &Ctx, id: Uuid, action: &str, params: Value) -> Result<Value, Error> {
        self.k.run_action(ctx, "MediaAsset", id, action, params).await
    }
    async fn status(&self, id: Uuid) -> String {
        self.get("MediaAsset", id).await["status"].as_str().unwrap().to_string()
    }
}

fn refused<V: std::fmt::Debug>(r: Result<V, Error>, kind: &str, says: &str) {
    let msg = format!("{r:?}");
    let ok = matches!(
        (&r, kind),
        (Err(Error::Invalid(_) | Error::Validation(_)), "invalid") | (Err(Error::Forbidden(_)), "forbidden") | (Err(Error::NotFound(_)), "not found")
    );
    assert!(ok && msg.contains(says), "expected {kind} saying {says:?}, got {msg}");
}

#[test]
fn a_brands_library() {
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().thread_stack_size(16 << 20).build().unwrap();
    rt.block_on(async { tokio::spawn(scenario()).await.unwrap() });
}

async fn scenario() {
    let url = std::env::var("FERRUM_DAM_TEST_DATABASE_URL").unwrap_or_else(|_| "postgres://ferrum:ferrum@localhost:5432/ferrum_dam_test".into());
    let db = Db::connect(DbConfig { url, max_connections: 16, ..Default::default() }).await.expect("connect");
    sqlx::raw_sql("drop schema if exists public cascade; create schema public;").execute(&db.pool).await.expect("reset");
    let mut packages = ferrum_domain::packages().expect("platform packages");
    packages.extend(ferrum_app_drive::packages().expect("Drive's packages"));
    packages.extend(ferrum_app_dam::packages().expect("app packages"));
    let catalog = Catalog::from_packages(packages).expect("catalog");
    let mut hooks = HookRegistry::default();
    ferrum_domain::register(&mut hooks);
    ferrum_app_drive::register(&mut hooks);
    ferrum_app_dam::register(&mut hooks);
    let settings = KernelSettings { secret_key: Some(SecretKey::new([7u8; 32])), ..Default::default() };
    let k = Kernel::boot(db, catalog, hooks, settings, None).await.expect("boot");
    k.set_file_store(Arc::new(MemFiles::default()));
    ferrum_domain::maintenance(&k).await.expect("partitions");
    let tenant = k
        .create_tenant(NewTenant {
            slug: "brand".into(),
            name: "Brand Co".into(),
            admin_email: "admin@brand.test".into(),
            admin_password: "Correct-Horse-9".into(),
            admin_name: None,
            packages: vec!["dam".into()],
            settings: Some(json!({"timezone": "UTC"})),
        })
        .await
        .expect("tenant");
    let session = k.login("brand", "admin@brand.test", "Correct-Horse-9").await.expect("login");
    let admin = k.load_ctx(tenant, session.user, Some(session.token_version), "it".into()).await.expect("ctx");
    let t = T { k: k.clone(), tenant, admin };
    t.insert(&t.admin, "DriveSettings", json!({"max_upload_mb": 2, "user_quota_mb": 100, "team_quota_mb": 1000, "trash_days": 30, "max_versions": 5})).await;
    let brand = t.user("brand@brand.test", &["Brand Manager", "Drive User"]).await;
    let maker = t.user("maker@brand.test", &["Media Contributor", "Drive User"]).await;
    let user = t.user("user@brand.test", &["Media User"]).await;
    let today = Utc::now().date_naive();

    // ───────────── files in Drive ─────────────
    let space = t.insert(&maker, "DriveSpace", json!({})).await;
    let (space_id, root) = (uid(&space["id"]), space["root_folder"].clone());
    let upload = |name: &'static str, data: Vec<u8>| {
        let k = k.clone();
        let maker = maker.clone();
        let root = root.clone();
        async move {
            let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
            let made =
                k.run_action(&maker, "DriveSpace", space_id, "upload", json!({"folder": root, "name": name, "content_base64": b64})).await.expect("uploaded");
            uid(&made["id"])
        }
    };
    let logo_file = upload("logo.png", png(1)).await;
    let photo_file = upload("press.png", png(2)).await;
    let stock_file = upload("stock.png", png(3)).await;

    // ───────────── assets and their rights ─────────────
    let collection = uid(&t.insert(&brand, "AssetCollection", json!({"collection_name": "Launch 2026"})).await["id"]);
    let asset = |title: &str, file: Uuid, license: &str, extra: Value| {
        let mut v = json!({"title": title, "drive_file": file.to_string(), "collection": collection.to_string(), "license": license});
        v.as_object_mut().unwrap().extend(extra.as_object().cloned().unwrap_or_default());
        v
    };
    let logo = t.insert(&maker, "MediaAsset", asset("Logo", logo_file, "Owned", json!({}))).await;
    assert_eq!(
        (logo["kind"].as_str(), logo["content_type"].as_str(), logo["size"].as_i64()),
        (Some("Image"), Some("image/png"), Some(600)),
        "read from the file"
    );
    assert!(logo["sha256"].as_str().is_some_and(|s| s.len() == 64));
    let logo = uid(&logo["id"]);
    refused(
        t.k.insert(&maker, "MediaAsset", obj(asset("Stock", stock_file, "Licensed", json!({"licensor": "Pics Ltd"})))).await,
        "invalid",
        "names its licensor and the last day",
    );
    refused(t.k.insert(&maker, "MediaAsset", obj(asset("CC", stock_file, "Creative Commons", json!({})))).await, "invalid", "names whom it credits");
    refused(
        t.k.insert(&maker, "MediaAsset", obj(asset("Stock", stock_file, "Licensed", json!({"usage_expires_on": (today + Duration::days(9)).to_string()}))))
            .await,
        "invalid",
        "names its licensor",
    );
    let last_day = today + Duration::days(20);
    let stock = uid(&t
        .insert(&maker, "MediaAsset", asset("Stock", stock_file, "Licensed", json!({"licensor": "Pics Ltd", "usage_expires_on": last_day.to_string()})))
        .await["id"]);
    let press = uid(&t.insert(&maker, "MediaAsset", asset("Press photo", photo_file, "Editorial Only", json!({}))).await["id"]);
    refused(t.k.insert(&maker, "MediaAsset", obj(asset("Ghost", Uuid::now_v7(), "Owned", json!({})))).await, "invalid", "");
    let binned = upload("old.png", png(4)).await;
    k.run_action(&maker, "DriveSpace", space_id, "trash", json!({"item": binned})).await.expect("in the trash");
    refused(t.k.insert(&maker, "MediaAsset", obj(asset("Binned", binned, "Owned", json!({})))).await, "invalid", "in the trash");

    // ───────────── review ─────────────
    for a in [logo, stock, press] {
        t.act(&maker, a, "submit_asset", Value::Null).await.unwrap();
    }
    refused(t.act(&maker, logo, "approve_asset", Value::Null).await, "forbidden", "a Brand Manager decides");
    refused(t.act(&brand, press, "reject_asset", Value::Null).await, "invalid", "says why");
    t.act(&brand, press, "reject_asset", json!({"note": "Crop the caption"})).await.unwrap();
    assert_eq!(
        (t.status(press).await, t.get("MediaAsset", press).await["review_note"].as_str().map(str::to_string)),
        ("Draft".into(), Some("Crop the caption".into()))
    );
    t.act(&maker, press, "submit_asset", Value::Null).await.unwrap();
    for a in [logo, stock, press] {
        t.act(&brand, a, "approve_asset", Value::Null).await.expect("approved");
    }
    assert_eq!(t.get("MediaAsset", logo).await["approved_version"], 1, "the file's version then");
    refused(
        t.k.update(&maker, "MediaAsset", stock, obj(json!({"usage_expires_on": (today + Duration::days(400)).to_string()})), None).await,
        "invalid",
        "keeps its file and rights",
    );
    t.k.update(&maker, "MediaAsset", logo, obj(json!({"tags": "logo brand mark"})), None).await.expect("its tags are not its rights");

    // ───────────── usage ─────────────
    let usage = |a: Uuid, channel: &str, on: chrono::NaiveDate| json!({"asset": a.to_string(), "channel": channel, "used_on": on.to_string(), "placement": "Home page"});
    let used = t.insert(&user, "AssetUsage", usage(logo, "Web", today)).await;
    assert_eq!(used["used_by"], "user@brand.test");
    t.insert(&user, "AssetUsage", usage(logo, "Advertising", today)).await;
    assert_eq!(t.get("MediaAsset", logo).await["usages"], 2);
    t.insert(&user, "AssetUsage", usage(press, "Web", today)).await;
    refused(t.k.insert(&user, "AssetUsage", obj(usage(press, "Advertising", today))).await, "invalid", "not used to sell");
    refused(t.k.insert(&user, "AssetUsage", obj(usage(press, "Social", today))).await, "invalid", "not used to sell");
    refused(t.k.insert(&user, "AssetUsage", obj(usage(stock, "Web", last_day + Duration::days(1)))).await, "invalid", "after the last day");
    t.insert(&user, "AssetUsage", usage(stock, "Print", last_day)).await;

    // ───────────── a new version, and expiry ─────────────
    let b64 = base64::engine::general_purpose::STANDARD.encode(png(9));
    k.run_action(&maker, "DriveSpace", space_id, "upload_version", json!({"file": logo_file, "content_base64": b64})).await.expect("a new logo");
    refused(t.k.insert(&user, "AssetUsage", obj(usage(logo, "Web", today))).await, "invalid", "changed since it was approved");
    ferrum_app_dam::maintenance_on(&k, today).await.unwrap();
    let back = t.get("MediaAsset", logo).await;
    assert_eq!(back["status"], "In Review", "back for review once its file changed");
    assert!(back["review_note"].as_str().unwrap().contains("since version 1"));
    t.act(&brand, logo, "approve_asset", Value::Null).await.unwrap();
    assert_eq!(t.get("MediaAsset", logo).await["approved_version"], 2);
    t.insert(&user, "AssetUsage", usage(logo, "Web", today)).await;
    ferrum_app_dam::maintenance_on(&k, last_day + Duration::days(1)).await.unwrap();
    assert_eq!(t.status(stock).await, "Expired");
    assert_eq!(t.status(press).await, "Approved", "no last day, no expiry");

    // ───────────── renditions, and what the file says ─────────────
    let launch_file = upload("launch.jpg", photo(1600, 1200, 90)).await;
    let mut clear = Vec::new();
    image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(500, 500, image::Rgba([200, 30, 30, 0])))
        .write_to(std::io::Cursor::new(&mut clear), image::ImageFormat::Png)
        .unwrap();
    let mark_file = upload("mark.png", clear).await;
    let brief_file = upload("brief.pdf", b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n1 0 obj << >> endobj\ntrailer << >>\n%%EOF\n".to_vec()).await;
    let launch = uid(&t.insert(&maker, "MediaAsset", asset("Launch photo", launch_file, "Owned", json!({}))).await["id"]);
    let mark = uid(&t.insert(&maker, "MediaAsset", asset("Mark", mark_file, "Owned", json!({}))).await["id"]);
    let brief = uid(&t.insert(&maker, "MediaAsset", asset("Brief", brief_file, "Owned", json!({}))).await["id"]);
    ferrum_app_dam::maintenance_on(&k, today).await.unwrap();
    let p = t.get("MediaAsset", launch).await;
    assert_eq!((p["width"].as_i64(), p["height"].as_i64()), (Some(1200), Some(1600)), "turned as the camera was held");
    assert_eq!(
        (p["taken"].as_str(), p["camera"].as_str(), p["file_credit"].as_str(), p["file_copyright"].as_str(), p["has_location"].as_bool()),
        (Some("2026-05-01 10:30:00"), Some("Canon EOS R6"), Some("Jo Doe"), Some("Brand Co"), Some(true)),
        "what its file says, the place itself not kept"
    );
    let thumbnail = file_id(&p["thumbnail"]);
    let (_, ctype, bytes) = t.k.read_file(&t.admin, thumbnail).await.unwrap();
    assert_eq!((ctype.as_str(), pixels(&bytes)), ("image/jpeg", (240, 320)));
    let (_, _, bytes) = t.k.read_file(&t.admin, file_id(&p["preview"])).await.unwrap();
    assert_eq!(pixels(&bytes), (960, 1280));
    let m = t.get("MediaAsset", mark).await;
    let (_, ctype, bytes) = t.k.read_file(&t.admin, file_id(&m["thumbnail"])).await.unwrap();
    assert_eq!((ctype.as_str(), pixels(&bytes), m["has_location"].as_bool()), ("image/png", (320, 320), Some(false)), "transparency kept");
    let b = t.get("MediaAsset", brief).await;
    assert!(b["thumbnail"].is_null() && b["rendition_note"].as_str().is_some_and(|n| n.contains("made of images")), "{b:?}");
    let l = t.get("MediaAsset", logo).await;
    assert!(l["rendition_note"].as_str().is_some_and(|n| n.contains("could not be read as an image")), "{l:?}");
    // Renditions are seen by whoever sees the asset.
    refused(t.k.read_file(&user, thumbnail).await, "not found", "");
    t.act(&maker, launch, "submit_asset", Value::Null).await.unwrap();
    t.act(&brand, launch, "approve_asset", Value::Null).await.unwrap();
    t.k.read_file(&user, thumbnail).await.expect("an approved asset's thumbnail");
    // A new version is looked at again, and the renditions of the one before go.
    let b64 = base64::engine::general_purpose::STANDARD.encode(photo(800, 600, 200));
    k.run_action(&maker, "DriveSpace", space_id, "upload_version", json!({"file": launch_file, "content_base64": b64})).await.expect("a new photo");
    ferrum_app_dam::maintenance_on(&k, today).await.unwrap();
    let again = t.get("MediaAsset", launch).await;
    assert_eq!((again["status"].as_str(), again["width"].as_i64(), again["height"].as_i64()), (Some("In Review"), Some(600), Some(800)));
    assert_ne!(again["thumbnail"], p["thumbnail"]);
    refused(t.k.read_file(&t.admin, thumbnail).await, "not found", "");

    // ───────────── who sees and does what ─────────────
    let seen = t.k.list(&user, "MediaAsset", &ListQuery::default()).await.unwrap().rows;
    let mut titles: Vec<&str> = seen.iter().map(|a| a["title"].as_str().unwrap()).collect();
    titles.sort();
    assert_eq!(titles, ["Logo", "Press photo"], "an asset user sees approved assets only");
    refused(t.k.insert(&user, "MediaAsset", obj(asset("Mine", logo_file, "Owned", json!({})))).await, "forbidden", "");
    t.act(&brand, press, "archive_asset", Value::Null).await.unwrap();
    refused(t.k.insert(&user, "AssetUsage", obj(usage(press, "Web", today))).await, "invalid", "not approved for use");
}
