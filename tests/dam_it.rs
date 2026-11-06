//! A brand's library (FERRUM-ECOSYSTEM-012) against a real PostgreSQL
//! (`FERRUM_DAM_TEST_DATABASE_URL`, default `postgres://ferrum:ferrum@localhost:5432/ferrum_dam_test`;
//! reset first), with Drive installed: files uploaded to Drive made assets whose type, size and
//! digest come from the file; licences naming their licensor and days; approval by a Brand Manager
//! of the file's version then; a new version sending the asset back for review; usages only of the
//! approved version within its days, editorial-only assets not used to sell; links handing an
//! approved asset outside, giving the version approved while they and the asset allow; expiry; renditions
//! and what a photo's file says of it, made again for a new version and seen by whoever sees the
//! asset; a PDF's XMP read, its keywords becoming its tags; a PDF's first page drawn as its
//! renditions; an office document's picture of its first page its renditions; the people and place a file shows and its licence's terms, approval waiting
//! for releases; previews of what a page shows by itself, for whoever sees the asset; and who sees
//! and does what.

use base64::Engine;
use chrono::{Duration, Utc};
use ferrum_kernel::secret::SecretKey;
use ferrum_kernel::tenant::NewTenant;
use ferrum_kernel::{Ctx, Db, DbConfig, Doc, Error, HookRegistry, Kernel, KernelSettings, ListQuery, SiteRequest};
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

/// A shared link opened by anyone, as the platform hands it to the app: the stored file it answers
/// with, or the error.
async fn open(k: &Kernel, tenant: Uuid, link: &str) -> Result<Uuid, Error> {
    let token = link.rsplit('/').next().unwrap_or_default();
    let site = k.hooks.site("dam").cloned().expect("the shared links' route");
    let req = SiteRequest {
        tenant,
        user: None,
        method: "GET".into(),
        path: format!("/s/{token}"),
        query: Default::default(),
        headers: Default::default(),
        body: vec![],
        client_ip: "203.0.113.20".into(),
    };
    let r = site.handle(k, req).await?;
    let file = r.headers.iter().find(|(name, _)| name == ferrum_kernel::hooks::STREAM_FILE_HEADER).map(|(_, v)| v.parse().unwrap());
    Ok(file.expect("answered with the stored file"))
}

/// `GET preview/{asset}` as the platform hands it to the app for `who` (nobody: not signed in): the
/// stored file it answers with, its type, and its headers.
async fn preview(k: &Kernel, tenant: Uuid, who: Option<&Ctx>, asset: Uuid) -> Result<(Uuid, String, Vec<(String, String)>), Error> {
    let site = k.hooks.site("dam").cloned().expect("the preview route");
    let req = SiteRequest {
        tenant,
        user: who.cloned(),
        method: "GET".into(),
        path: format!("/preview/{asset}"),
        query: Default::default(),
        headers: Default::default(),
        body: vec![],
        client_ip: "203.0.113.30".into(),
    };
    let r = site.handle(k, req).await?;
    let file = r.headers.iter().find(|(name, _)| name == ferrum_kernel::hooks::STREAM_FILE_HEADER).map(|(_, v)| v.parse().unwrap());
    Ok((file.expect("answered with the stored file"), r.content_type, r.headers))
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

    // ───────────── handed outside, by a link ─────────────
    let share = |recipient: &str, channel: &str, until: chrono::NaiveDate, downloads: Option<i64>| json!({"recipient": recipient, "channel": channel, "expires_on": until.to_string(), "max_downloads": downloads});
    refused(t.act(&maker, logo, "share_asset", share("Daily Times", "Web", today + Duration::days(7), None)).await, "forbidden", "a Brand Manager shares");
    refused(t.act(&brand, press, "share_asset", share("Ad agency", "Advertising", today + Duration::days(7), None)).await, "invalid", "not used to sell");
    refused(t.act(&brand, stock, "share_asset", share("Printer", "Print", last_day + Duration::days(1), None)).await, "invalid", "after the last day");
    refused(t.act(&brand, logo, "share_asset", share("Daily Times", "Web", today - Duration::days(1), None)).await, "invalid", "today or later");
    let usages = t.get("MediaAsset", logo).await["usages"].as_i64().unwrap();
    let shared = t.act(&brand, logo, "share_asset", share("Daily Times", "Web", today + Duration::days(7), Some(2))).await.expect("shared");
    let link = shared["link"].as_str().unwrap().to_string();
    assert!(link.contains("/api/site/brand/dam/s/") && shared["note"].as_str().is_some_and(|n| n.contains("once")), "{shared}");
    assert_eq!(t.get("MediaAsset", logo).await["usages"].as_i64(), Some(usages + 1), "a use like any other");
    let kept = t.k.list(&brand, "AssetShare", &ListQuery::default()).await.unwrap().rows;
    let token = link.rsplit('/').next().unwrap();
    assert!(kept.len() == 1 && kept[0]["recipient"] == "Daily Times" && !kept[0].values().any(|v| v.as_str() == Some(token)), "only its digest is kept");
    // It gives the version approved, as the stored file, for its downloads.
    let v1: Option<Option<String>> = {
        let mut tx = k.db.tenant_tx(tenant).await.unwrap();
        let v = sqlx::query_scalar("select stored_file from e_drive_file_version where tenant_id = $1 and file = $2 and version_no = 1")
            .bind(tenant)
            .bind(logo_file)
            .fetch_optional(&mut *tx)
            .await
            .unwrap();
        tx.commit().await.unwrap();
        v
    };
    let v1: Uuid = v1.flatten().unwrap().parse().unwrap();
    assert_eq!(open(&k, tenant, &link).await.unwrap(), v1);
    assert_eq!(open(&k, tenant, &link).await.unwrap(), v1);
    assert!(matches!(open(&k, tenant, &link).await, Err(Error::NotFound(_))), "two downloads, no more");
    assert!(matches!(open(&k, tenant, "/api/site/brand/dam/s/not-a-token").await, Err(Error::NotFound(_))));
    // Stopped, it is gone; another stays until the asset changes.
    let again =
        t.act(&brand, logo, "share_asset", share("Printer", "Print", today + Duration::days(7), None)).await.unwrap()["link"].as_str().unwrap().to_string();
    let second =
        t.k.list(
            &brand,
            "AssetShare",
            &ListQuery { filters: ferrum_kernel::query::parse_filters(&json!({"recipient": "Printer"})).unwrap(), ..Default::default() },
        )
        .await
        .unwrap()
        .rows;
    let printer =
        t.act(&brand, press, "share_asset", share("Printer", "Web", today + Duration::days(3), None)).await.unwrap()["link"].as_str().unwrap().to_string();
    t.k.run_action(&brand, "AssetShare", uid(&second[0]["id"]), "revoke_share", Value::Null).await.expect("stopped");
    assert!(matches!(open(&k, tenant, &again).await, Err(Error::NotFound(_))), "stopped");
    open(&k, tenant, &printer).await.expect("another still works");

    // ───────────── a new version, and expiry ─────────────
    let before =
        t.act(&brand, logo, "share_asset", share("Radio", "Web", today + Duration::days(3), None)).await.unwrap()["link"].as_str().unwrap().to_string();
    assert_eq!(open(&k, tenant, &before).await.unwrap(), v1);
    let b64 = base64::engine::general_purpose::STANDARD.encode(png(9));
    k.run_action(&maker, "DriveSpace", space_id, "upload_version", json!({"file": logo_file, "content_base64": b64})).await.expect("a new logo");
    refused(t.k.insert(&user, "AssetUsage", obj(usage(logo, "Web", today))).await, "invalid", "changed since it was approved");
    assert!(matches!(open(&k, tenant, &before).await, Err(Error::NotFound(_))), "its file changed since it was approved: nothing until it is again");
    ferrum_app_dam::maintenance_on(&k, today).await.unwrap();
    let back = t.get("MediaAsset", logo).await;
    assert_eq!(back["status"], "In Review", "back for review once its file changed");
    assert!(back["review_note"].as_str().unwrap().contains("since version 1"));
    t.act(&brand, logo, "approve_asset", Value::Null).await.unwrap();
    assert_eq!(t.get("MediaAsset", logo).await["approved_version"], 2);
    let v2 = open(&k, tenant, &before).await.expect("approved again, the link gives the version approved now");
    assert_ne!(v2, v1);
    t.insert(&user, "AssetUsage", usage(logo, "Web", today)).await;
    ferrum_app_dam::maintenance_on(&k, last_day + Duration::days(1)).await.unwrap();
    assert_eq!(t.status(stock).await, "Expired");
    // An asset archived: its links give nothing more.
    let archived_link =
        t.act(&brand, press, "share_asset", share("Archive test", "Web", today + Duration::days(3), None)).await.unwrap()["link"].as_str().unwrap().to_string();
    open(&k, tenant, &archived_link).await.expect("approved, it works");
    assert_eq!(t.status(press).await, "Approved", "no last day, no expiry");

    // ───────────── renditions, and what the file says ─────────────
    let launch_file = upload("launch.jpg", photo(1600, 1200, 90)).await;
    let mut clear = Vec::new();
    image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(500, 500, image::Rgba([200, 30, 30, 0])))
        .write_to(std::io::Cursor::new(&mut clear), image::ImageFormat::Png)
        .unwrap();
    let mark_file = upload("mark.png", clear).await;
    // A PDF whose XMP names its title, creator and keywords.
    let xmp = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/"><rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"><rdf:Description xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:title><rdf:Alt><rdf:li xml:lang="x-default">Spring brief</rdf:li></rdf:Alt></dc:title><dc:creator><rdf:Seq><rdf:li>Jo Doe</rdf:li></rdf:Seq></dc:creator><dc:subject><rdf:Bag><rdf:li>spring</rdf:li><rdf:li>brief</rdf:li></rdf:Bag></dc:subject></rdf:Description></rdf:RDF></x:xmpmeta>"#;
    let mut pdf = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n1 0 obj << /Type /Metadata /Subtype /XML >> stream\n".to_vec();
    pdf.extend_from_slice(xmp.as_bytes());
    pdf.extend_from_slice(b"\nendstream endobj\ntrailer << >>\n%%EOF\n");
    let brief_file = upload("brief.pdf", pdf).await;
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
    // A PDF with no pages to draw (only its XMP) says so.
    assert!(b["thumbnail"].is_null() && b["rendition_note"].as_str().is_some_and(|n| n.contains("PDF") || n.contains("no pages")), "{b:?}");
    assert_eq!(
        (b["file_title"].as_str(), b["file_credit"].as_str(), b["file_keywords"].as_str(), b["tags"].as_str()),
        (Some("Spring brief"), Some("Jo Doe"), Some("spring, brief"), Some("spring, brief")),
        "what its XMP says, its keywords its tags as it had none: {b:?}"
    );
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
    // An office document's picture of its first page makes its renditions; its size is not the picture's.
    let mut page = Vec::new();
    image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(400, 640, image::Rgb([240, 240, 240])))
        .write_to(std::io::Cursor::new(&mut page), image::ImageFormat::Png)
        .unwrap();
    let mut odt = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let stored = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (name, content) in [("mimetype", &b"application/vnd.oasis.opendocument.text"[..]), ("Thumbnails/thumbnail.png", &page[..])] {
        odt.start_file(name, stored).unwrap();
        std::io::Write::write_all(&mut odt, content).unwrap();
    }
    let spec_file = upload("spec.odt", odt.finish().unwrap().into_inner()).await;
    let spec = uid(&t.insert(&maker, "MediaAsset", asset("Spec", spec_file, "Owned", json!({}))).await["id"]);
    // A video, as its file begins.
    let mut mp4 = b"\0\0\0\x18ftypmp42\0\0\0\0mp42isom".to_vec();
    mp4.resize(4096, 0);
    let clip_file = upload("clip.mp4", mp4).await;
    let clip = uid(&t.insert(&maker, "MediaAsset", asset("Clip", clip_file, "Owned", json!({}))).await["id"]);
    // A PDF of one page, 300 by 600 points: its first page drawn makes its renditions.
    let flyer_file = upload("flyer.pdf", one_page_pdf(300, 600)).await;
    let flyer = uid(&t.insert(&maker, "MediaAsset", asset("Flyer", flyer_file, "Owned", json!({}))).await["id"]);
    ferrum_app_dam::maintenance_on(&k, today).await.unwrap();
    let d = t.get("MediaAsset", spec).await;
    let (_, ctype, bytes) = t.k.read_file(&t.admin, file_id(&d["thumbnail"])).await.unwrap();
    assert_eq!((ctype.as_str(), pixels(&bytes)), ("image/jpeg", (200, 320)), "{d:?}");
    assert!(d["preview"].is_string() && d["width"].is_null() && d["height"].is_null() && d["rendition_note"].is_null(), "{d:?}");
    let c = t.get("MediaAsset", clip).await;
    assert!(c["thumbnail"].is_null() && c["rendition_note"].as_str().is_some_and(|n| n.ends_with("this file is video")), "{c:?}");
    let fl = t.get("MediaAsset", flyer).await;
    let (_, _, bytes) = t.k.read_file(&t.admin, file_id(&fl["thumbnail"])).await.unwrap();
    assert_eq!(pixels(&bytes), (160, 320), "{fl:?}");
    let (_, _, bytes) = t.k.read_file(&t.admin, file_id(&fl["preview"])).await.unwrap();
    assert_eq!(pixels(&bytes), (640, 1280));
    assert!(fl["width"].is_null() && fl["rendition_note"].is_null(), "a page's picture says nothing of the file's size: {fl:?}");

    // The people and the place a file shows, and its licence's terms: approved only once releases are held.
    let said = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/"><rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"><rdf:Description xmlns:Iptc4xmpExt="http://iptc.org/std/Iptc4xmpExt/2008-02-29/" xmlns:xmpRights="http://ns.adobe.com/xap/1.0/rights/"><Iptc4xmpExt:PersonInImage><rdf:Bag><rdf:li>Ana Rao</rdf:li></rdf:Bag></Iptc4xmpExt:PersonInImage><Iptc4xmpExt:LocationShown><rdf:Bag><rdf:li rdf:parseType="Resource"><Iptc4xmpExt:City>Sydney</Iptc4xmpExt:City></rdf:li></rdf:Bag></Iptc4xmpExt:LocationShown><xmpRights:UsageTerms><rdf:Alt><rdf:li xml:lang="x-default">Web only</rdf:li></rdf:Alt></xmpRights:UsageTerms></rdf:Description></rdf:RDF></x:xmpmeta>"#;
    let mut team = b"%PDF-1.4\n1 0 obj << /Type /Metadata /Subtype /XML >> stream\n".to_vec();
    team.extend_from_slice(said.as_bytes());
    team.extend_from_slice(b"\nendstream endobj\ntrailer << >>\n%%EOF\n");
    let team_file = upload("team.pdf", team).await;
    let team = uid(&t.insert(&maker, "MediaAsset", asset("Team", team_file, "Owned", json!({}))).await["id"]);
    ferrum_app_dam::maintenance_on(&k, today).await.unwrap();
    let tm = t.get("MediaAsset", team).await;
    assert_eq!(
        (tm["file_people"].as_str(), tm["file_place"].as_str(), tm["file_usage_terms"].as_str()),
        (Some("Ana Rao"), Some("Sydney"), Some("Web only")),
        "{tm:?}"
    );
    t.act(&maker, team, "submit_asset", Value::Null).await.unwrap();
    refused(t.act(&brand, team, "approve_asset", Value::Null).await, "invalid", "hold their releases");
    t.act(&brand, team, "reject_asset", json!({"note": "Releases first"})).await.unwrap();
    t.k.update(&maker, "MediaAsset", team, obj(json!({"people_released": true})), None).await.expect("releases held");
    t.act(&maker, team, "submit_asset", Value::Null).await.unwrap();
    t.act(&brand, team, "approve_asset", Value::Null).await.expect("approved once releases are held");
    t.act(&brand, team, "archive_asset", Value::Null).await.unwrap();

    // ───────────── previews ─────────────
    // Whoever sees an asset sees its file in the page, for the kinds a page shows by itself.
    let (file, ctype, headers) = preview(&k, tenant, Some(&maker), brief).await.expect("a PDF's preview");
    let (_, _, bytes) = t.k.read_file(&t.admin, file).await.unwrap();
    assert!(ctype == "application/pdf" && bytes.starts_with(b"%PDF"), "{ctype}");
    let header = |name: &str| headers.iter().find(|(n, _)| n == name).map(|(_, v)| v.as_str());
    assert_eq!(
        (header("content-disposition"), header("x-content-type-options"), header("cache-control")),
        (Some("inline; filename=\"brief.pdf\""), Some("nosniff"), Some("private, no-store"))
    );
    assert_eq!(preview(&k, tenant, Some(&maker), clip).await.expect("a video's preview").1, "video/mp4");
    refused(preview(&k, tenant, Some(&maker), spec).await, "not found", "a preview of a file of type application/vnd.oasis.opendocument.text");
    // Not to someone who does not see the asset, nor to nobody.
    refused(preview(&k, tenant, Some(&user), brief).await, "not found", "");
    assert!(matches!(preview(&k, tenant, None, brief).await, Err(Error::Unauthorized(_))));

    // ───────────── who sees and does what ─────────────
    let seen = t.k.list(&user, "MediaAsset", &ListQuery::default()).await.unwrap().rows;
    let mut titles: Vec<&str> = seen.iter().map(|a| a["title"].as_str().unwrap()).collect();
    titles.sort();
    assert_eq!(titles, ["Logo", "Press photo"], "an asset user sees approved assets only");
    refused(t.k.insert(&user, "MediaAsset", obj(asset("Mine", logo_file, "Owned", json!({})))).await, "forbidden", "");
    t.act(&brand, press, "archive_asset", Value::Null).await.unwrap();
    refused(t.k.insert(&user, "AssetUsage", obj(usage(press, "Web", today))).await, "invalid", "not approved for use");
    assert!(matches!(open(&k, tenant, &archived_link).await, Err(Error::NotFound(_))), "archived, its links give nothing");
}

/// A PDF of one page, `w` by `h` points, filled blue; its cross-reference worked out.
fn one_page_pdf(w: u32, h: u32) -> Vec<u8> {
    let content = format!("0 0 1 rg 0 0 {w} {h} re f");
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
        format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {w} {h}] /Contents 4 0 R >>"),
        format!("<< /Length {} >>\nstream\n{content}\nendstream", content.len()),
    ];
    let mut out = b"%PDF-1.4\n".to_vec();
    let mut offsets = vec![];
    for (i, o) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend(format!("{} 0 obj\n{o}\nendobj\n", i + 1).bytes());
    }
    let xref = out.len();
    out.extend(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).bytes());
    for o in offsets {
        out.extend(format!("{o:010} 00000 n \n").bytes());
    }
    out.extend(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objects.len() + 1).bytes());
    out
}
