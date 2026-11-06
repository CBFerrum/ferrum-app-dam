//! Renditions, and what a file says of itself. Each round, the assets whose file changed since they
//! were last looked at (new assets, and new versions) are read, a few at a time:
//!
//! * an image gets a thumbnail (320 pixels on its longest side) and a preview (1280 at most), turned
//!   the way the camera was held, as JPEG, or as PNG when it has transparency;
//! * its size in pixels, when it was taken (in the camera's own time), the camera, and the creator
//!   and copyright it names are kept on the asset. Where it was taken never is: the asset only says
//!   the file records it, so the place can be removed before the file goes out.
//! * any file's XMP, and a JPEG's IPTC, give its title, caption and keywords, and the creator and
//!   copyright EXIF leaves unsaid (`embedded`); its keywords become its tags when it has none.
//!
//! An office document's renditions are made of the picture of its first page it carries, when it
//! carries one. Other files, images over 40 megapixels or 50 MB, and files that cannot be read as
//! images keep a note saying why they have no renditions. A new version is looked at again, and the
//! renditions of the one before are removed.

use crate::obj;
use exif::{In, Tag};
use ferrum_kernel::{Ctx, Error, Kernel, Result};
use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat, ImageReader, Limits};
use serde_json::{Value, json};
use std::io::Cursor;
use uuid::Uuid;

/// Assets looked at in a round, at most.
const PER_ROUND: i64 = 10;
/// The longest side of a thumbnail, and of a preview, in pixels.
const THUMBNAIL: u32 = 320;
const PREVIEW: u32 = 1280;
/// The largest images renditions are made of.
const MAX_BYTES: i64 = 50 << 20;
const MAX_PIXELS: u64 = 40_000_000;

/// What a file's EXIF says of it, and its XMP or IPTC.
#[derive(Debug, Default, PartialEq)]
pub(crate) struct Facts {
    pub taken: Option<String>,
    pub camera: Option<String>,
    pub credit: Option<String>,
    pub copyright: Option<String>,
    pub orientation: Option<u8>,
    pub located: bool,
    pub title: Option<String>,
    pub caption: Option<String>,
    pub keywords: Vec<String>,
}

/// When the file was taken, with what, by whom, under whose copyright, how the camera was held, and
/// whether it records where (EXIF); its title, caption and keywords (XMP or IPTC), which also give
/// the creator and copyright EXIF leaves unsaid.
pub(crate) fn facts(bytes: &[u8]) -> Facts {
    let said = crate::embedded::said(bytes);
    let f = exif_facts(bytes);
    Facts {
        credit: f.credit.or(said.creator).or(said.credit),
        copyright: f.copyright.or(said.copyright),
        title: said.title,
        caption: said.caption,
        keywords: said.keywords,
        ..f
    }
}

fn exif_facts(bytes: &[u8]) -> Facts {
    let Ok(exif) = exif::Reader::new().read_from_container(&mut Cursor::new(bytes)) else { return Facts::default() };
    let ascii = |tag: Tag| match exif.get_field(tag, In::PRIMARY).map(|f| &f.value) {
        Some(exif::Value::Ascii(v)) => v.first().cloned(),
        _ => None,
    };
    let text =
        |tag: Tag| ascii(tag).map(|b| String::from_utf8_lossy(&b).trim_matches(|c: char| c.is_whitespace() || c == '\0').to_string()).filter(|t| !t.is_empty());
    let taken = ascii(Tag::DateTimeOriginal)
        .and_then(|b| exif::DateTime::from_ascii(&b).ok())
        .map(|d| format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", d.year, d.month, d.day, d.hour, d.minute, d.second));
    let camera = match (text(Tag::Make), text(Tag::Model)) {
        (Some(make), Some(model)) if model.starts_with(&make) => Some(model),
        (Some(make), Some(model)) => Some(format!("{make} {model}")),
        (make, model) => make.or(model),
    };
    Facts {
        taken,
        camera,
        credit: text(Tag::Artist),
        copyright: text(Tag::Copyright),
        orientation: exif.get_field(Tag::Orientation, In::PRIMARY).and_then(|f| f.value.get_uint(0)).and_then(|o| u8::try_from(o).ok()),
        located: exif.get_field(Tag::GPSLatitude, In::PRIMARY).is_some() || exif.get_field(Tag::GPSLongitude, In::PRIMARY).is_some(),
        ..Facts::default()
    }
}

/// An image's renditions, and its size as it is shown.
#[derive(Debug)]
pub(crate) struct Rendered {
    pub width: u32,
    pub height: u32,
    pub thumbnail: Vec<u8>,
    pub preview: Vec<u8>,
    /// PNG (it has transparency), else JPEG.
    pub png: bool,
}

/// Renditions of `bytes`, turned by the EXIF `orientation`; why not, as a note, when there are none.
pub(crate) fn render(bytes: &[u8], orientation: Option<u8>) -> std::result::Result<Rendered, String> {
    let unread = |e: image::ImageError| format!("could not be read as an image: {e}");
    let reader = || {
        let mut r = ImageReader::new(Cursor::new(bytes)).with_guessed_format().map_err(|e| format!("could not be read: {e}"))?;
        let mut limits = Limits::default();
        limits.max_alloc = Some(512 << 20);
        r.limits(limits);
        Ok::<_, String>(r)
    };
    let (w, h) = reader()?.into_dimensions().map_err(unread)?;
    if u64::from(w) * u64::from(h) > MAX_PIXELS {
        return Err(format!("is {w} by {h} pixels, more than renditions are made of ({} megapixels)", MAX_PIXELS / 1_000_000));
    }
    let mut img = reader()?.decode().map_err(unread)?;
    if let Some(o) = orientation.and_then(image::metadata::Orientation::from_exif) {
        img.apply_orientation(o);
    }
    let png = img.color().has_alpha();
    let fit = |side: u32, fast: bool| match (img.width() <= side && img.height() <= side, fast) {
        (true, _) => img.clone(),
        (false, true) => img.thumbnail(side, side),
        (false, false) => img.resize(side, side, FilterType::Triangle),
    };
    let encode = |r: DynamicImage| {
        let mut out = Vec::new();
        let done = if png {
            DynamicImage::ImageRgba8(r.to_rgba8()).write_to(Cursor::new(&mut out), ImageFormat::Png)
        } else {
            DynamicImage::ImageRgb8(r.to_rgb8()).write_with_encoder(image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 82))
        };
        done.map(|_| out).map_err(|e| format!("its renditions could not be written: {e}"))
    };
    Ok(Rendered { width: img.width(), height: img.height(), thumbnail: encode(fit(THUMBNAIL, true))?, preview: encode(fit(PREVIEW, false))?, png })
}

/// A stored file's id from its address (`/files/{id}`).
fn stored(url: Option<&str>) -> Option<Uuid> {
    url?.strip_prefix("/files/")?.parse().ok()
}

/// One round in a workspace: renditions and facts for the assets whose file changed since they were
/// last looked at. A store that is unavailable stops the round, to be tried again; any other file
/// that cannot be read keeps a note.
pub(crate) async fn round(k: &Kernel, ctx: &Ctx) -> Result<usize> {
    let mut tx = k.db.tenant_tx(ctx.tenant).await?;
    let due: Vec<(Uuid, Option<String>, Option<String>, Option<String>, Option<String>, Option<i64>, Option<String>, Option<String>)> = sqlx::query_as(
        "select a.id, f.sha256, v.stored_file, a.thumbnail, a.preview, f.size, f.file_type, a.tags from e_media_asset a \
         join e_drive_file f on f.tenant_id = a.tenant_id and f.id = a.drive_file \
         left join e_drive_file_version v on v.tenant_id = f.tenant_id and v.file = f.id and v.version_no = f.version_no \
         where a.tenant_id = $1 and a.renditions_of is distinct from f.sha256 order by a.id limit $2",
    )
    .bind(ctx.tenant)
    .bind(PER_ROUND)
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;
    let n = due.len();
    for (asset, sha, object, old_thumbnail, old_preview, size, file_type, tags) in due {
        let mut values = json!({"renditions_of": sha, "width": null, "height": null, "taken": null, "camera": null, "file_credit": null,
                                "file_copyright": null, "file_title": null, "file_caption": null, "file_keywords": null, "has_location": false,
                                "thumbnail": null, "preview": null, "rendition_note": null});
        let note = |v: &mut Value, why: String| v["rendition_note"] = json!(why);
        let not_image = (file_type.as_deref() != Some("Image")).then(|| {
            format!(
                "Renditions are made of images, and of documents that carry a picture of their first page; this file is {}",
                file_type.as_deref().map(str::to_lowercase).unwrap_or_else(|| "of no known kind".into())
            )
        });
        // Any file is read for what it says of itself; only an image is rendered.
        match object.as_deref().and_then(|o| o.parse::<Uuid>().ok()) {
            None => note(&mut values, not_image.unwrap_or_else(|| "Its file's content is missing".into())),
            Some(_) if size.unwrap_or(0) > MAX_BYTES => {
                note(&mut values, not_image.unwrap_or_else(|| format!("It is larger than {} MB, more than renditions are made of", MAX_BYTES >> 20)))
            }
            Some(object) => match k.read_file(ctx, object).await {
                Err(e @ Error::Unavailable(_)) => return Err(e),
                Err(e) => note(&mut values, format!("Its file could not be read: {e}")),
                Ok((_, _, bytes)) => {
                    let f = facts(&bytes);
                    values["taken"] = json!(f.taken);
                    values["camera"] = json!(f.camera);
                    values["file_credit"] = json!(f.credit);
                    values["file_copyright"] = json!(f.copyright);
                    values["file_title"] = json!(f.title);
                    values["file_caption"] = json!(f.caption);
                    values["has_location"] = json!(f.located);
                    if !f.keywords.is_empty() {
                        let words = f.keywords.join(", ");
                        // Its keywords are its tags when it has none.
                        if tags.as_deref().is_none_or(|t| t.trim().is_empty()) {
                            values["tags"] = json!(words);
                        }
                        values["file_keywords"] = json!(words);
                    }
                    let orientation = f.orientation;
                    // Decoding and scaling hold a thread: not the one answering requests.
                    // A document's picture of its first page, when it carries one.
                    let picture = not_image.as_ref().and_then(|_| crate::preview::embedded_thumbnail(&bytes));
                    let image = not_image.is_none();
                    let made = match (not_image, picture) {
                        (Some(_), Some(picture)) => tokio::task::spawn_blocking(move || render(&picture, None))
                            .await
                            .map_err(|e| Error::internal(e.to_string()))?
                            .map_err(|why| format!("Its picture of its first page {why}")),
                        (Some(why), None) => Err(why),
                        (None, _) => tokio::task::spawn_blocking(move || render(&bytes, orientation))
                            .await
                            .map_err(|e| Error::internal(e.to_string()))?
                            .map_err(|why| format!("It {why}")),
                    };
                    match made {
                        Err(why) => note(&mut values, why),
                        Ok(r) => {
                            let (ext, ctype) = if r.png { ("png", "image/png") } else { ("jpg", "image/jpeg") };
                            let on = Some(("MediaAsset", asset));
                            let thumbnail = k.store_file(ctx, on, &format!("thumbnail.{ext}"), ctype, &r.thumbnail).await?;
                            let preview = k.store_file(ctx, on, &format!("preview.{ext}"), ctype, &r.preview).await?;
                            // A document's picture says nothing of the document's size.
                            if image {
                                values["width"] = json!(r.width);
                                values["height"] = json!(r.height);
                            }
                            values["thumbnail"] = thumbnail["url"].clone();
                            values["preview"] = preview["url"].clone();
                        }
                    }
                }
            },
        }
        let mut tx = k.db.tenant_tx(ctx.tenant).await?;
        k.set_values_tx(ctx, &mut tx, "MediaAsset", asset, obj(values)).await?;
        tx.commit().await?;
        // The renditions of the version before, no longer the asset's.
        for old in [old_thumbnail, old_preview].iter().filter_map(|u| stored(u.as_deref())) {
            let _ = k.remove_file(ctx, old).await;
        }
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage, Rgba, RgbaImage};

    fn jpeg(w: u32, h: u32) -> Vec<u8> {
        let mut out = Vec::new();
        DynamicImage::ImageRgb8(RgbImage::from_fn(w, h, |x, y| Rgb([(x % 256) as u8, (y % 256) as u8, 90])))
            .write_with_encoder(image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 90))
            .unwrap();
        out
    }

    fn size(bytes: &[u8]) -> (u32, u32) {
        ImageReader::new(Cursor::new(bytes)).with_guessed_format().unwrap().into_dimensions().unwrap()
    }

    #[test]
    fn renditions_of_an_image() {
        let r = render(&jpeg(2000, 1000), None).unwrap();
        assert_eq!((r.width, r.height, r.png), (2000, 1000, false));
        assert_eq!((size(&r.thumbnail), size(&r.preview)), ((320, 160), (1280, 640)), "within their sides, as it is shaped");
        // Held on its side (EXIF orientation 6), it is shown turned.
        let r = render(&jpeg(400, 200), Some(6)).unwrap();
        assert_eq!((r.width, r.height, size(&r.thumbnail), size(&r.preview)), (200, 400, (160, 320), (200, 400)), "never made larger");
        let mut out = Vec::new();
        DynamicImage::ImageRgba8(RgbaImage::from_pixel(64, 64, Rgba([0, 0, 0, 0]))).write_to(Cursor::new(&mut out), ImageFormat::Png).unwrap();
        let r = render(&out, None).unwrap();
        assert!(r.png && r.thumbnail.starts_with(b"\x89PNG"), "transparency kept");
        assert!(render(b"\x89PNG\r\n\x1a\nnot really", None).unwrap_err().starts_with("could not be read as an image"));
        assert_eq!(facts(&jpeg(10, 10)), Facts::default(), "no EXIF, nothing said");
    }

    #[test]
    fn files_addressed_by_id() {
        let id = Uuid::now_v7();
        assert_eq!(stored(Some(&format!("/files/{id}"))), Some(id));
        assert_eq!((stored(Some("/files/nope")), stored(Some("https://elsewhere.test/x")), stored(None)), (None, None, None));
    }
}
