//! Previews. Whoever may see an asset sees its file in the page: `GET /api/app/dam/preview/{asset}`
//! answers the file of its current version, read with the person's own rights, for the kinds a
//! browser shows by itself and safely (PDF, video, audio and pictures, never a page or SVG that
//! could run in this workspace's origin). An office document's thumbnail comes from the picture of
//! its first page it carries (ODF always does; OOXML when saved with one).

use ferrum_domain::util::id;
use ferrum_kernel::{Ctx, Error, Kernel, Result, SiteRequest, SiteResponse};
use std::io::{Cursor, Read};
use uuid::Uuid;

/// The kinds of file a page shows by itself, without running anything of theirs.
pub(crate) fn previewable(content_type: &str) -> bool {
    let t = content_type.split(';').next().unwrap_or_default().trim().to_ascii_lowercase();
    matches!(
        t.as_str(),
        "application/pdf"
            | "video/mp4"
            | "video/webm"
            | "video/ogg"
            | "video/quicktime"
            | "audio/mpeg"
            | "audio/mp4"
            | "audio/ogg"
            | "audio/wav"
            | "audio/webm"
            | "audio/flac"
            | "audio/aac"
            | "image/png"
            | "image/jpeg"
            | "image/gif"
            | "image/webp"
            | "image/avif"
    )
}

/// `GET preview/{asset}`: the asset's current file, for whoever may see the asset, when a page shows
/// its kind by itself.
pub(crate) async fn preview(k: &Kernel, req: &SiteRequest, asset: &str) -> Result<SiteResponse> {
    let user = req.user.as_ref().ok_or_else(|| Error::Unauthorized("sign in first".into()))?;
    let asset: Uuid = asset.parse().map_err(|_| Error::NotFound("asset".into()))?;
    // The person's own rights: an asset they cannot see is not found.
    let doc = k.get(user, "MediaAsset", asset, false).await?;
    let file = id(&doc, "drive_file").ok_or_else(|| Error::NotFound("its file".into()))?;
    let ctx = Ctx::system(user.tenant, k.registry.schema(user.tenant).await?);
    let mut tx = k.db.tenant_tx(user.tenant).await?;
    let stored: Option<Option<String>> = sqlx::query_scalar(
        "select v.stored_file from e_drive_file f join e_drive_file_version v on v.tenant_id = f.tenant_id and v.file = f.id and v.version_no = f.version_no \
         where f.tenant_id = $1 and f.id = $2",
    )
    .bind(user.tenant)
    .bind(file)
    .fetch_optional(&mut *tx)
    .await?;
    tx.commit().await?;
    let object: Uuid = stored.flatten().and_then(|o| o.parse().ok()).ok_or_else(|| Error::NotFound("its file's content".into()))?;
    let info = k.file_info(&ctx, object).await?;
    if !previewable(&info.content_type) {
        return Err(Error::NotFound(format!("a preview of a file of type {}", info.content_type)));
    }
    let name: String = info.name.chars().filter(|c| !c.is_control() && *c != '"' && *c != '\\').collect();
    let headers = vec![
        ("content-disposition".to_string(), format!("inline; filename=\"{name}\"")),
        ("cache-control".to_string(), "private, no-store".to_string()),
        ("x-content-type-options".to_string(), "nosniff".to_string()),
        ("cross-origin-resource-policy".to_string(), "same-origin".to_string()),
    ];
    Ok(SiteResponse::file(200, &info.content_type, headers, object))
}

/// The picture of its first page an office document carries: ODF's `Thumbnails/thumbnail.png`, or
/// OOXML's `docProps/thumbnail` in JPEG or PNG; at most 10 MB of it is read.
pub(crate) fn embedded_thumbnail(bytes: &[u8]) -> Option<Vec<u8>> {
    if !bytes.starts_with(b"PK\x03\x04") {
        return None;
    }
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).ok()?;
    for name in ["Thumbnails/thumbnail.png", "docProps/thumbnail.jpeg", "docProps/thumbnail.jpg", "docProps/thumbnail.png"] {
        let Ok(entry) = zip.by_name(name) else { continue };
        let mut out = Vec::new();
        entry.take(10 << 20).read_to_end(&mut out).ok()?;
        return (!out.is_empty()).then_some(out);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// A zip of `files` (name, content), stored.
    pub(crate) fn zipped(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut w = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        for (name, content) in files {
            w.start_file(*name, options).unwrap();
            w.write_all(content).unwrap();
        }
        w.finish().unwrap().into_inner()
    }

    #[test]
    fn what_a_page_shows_by_itself() {
        assert!(["application/pdf", "video/mp4", "audio/mpeg", "image/png", "Image/JPEG; charset=binary"].iter().all(|t| previewable(t)));
        assert!(!["text/html", "image/svg+xml", "application/xhtml+xml", "application/zip", "text/plain", ""].iter().any(|t| previewable(t)));
    }

    #[test]
    fn office_documents_carry_their_first_page() {
        let odt = zipped(&[("mimetype", b"application/vnd.oasis.opendocument.text"), ("Thumbnails/thumbnail.png", b"\x89PNG picture")]);
        assert_eq!(embedded_thumbnail(&odt).as_deref(), Some(&b"\x89PNG picture"[..]));
        let docx = zipped(&[("[Content_Types].xml", b"<Types/>"), ("docProps/thumbnail.jpeg", b"\xff\xd8 picture")]);
        assert_eq!(embedded_thumbnail(&docx).as_deref(), Some(&b"\xff\xd8 picture"[..]));
        let without = zipped(&[("word/document.xml", b"<w:document/>"), ("docProps/thumbnail.wmf", b"metafile")]);
        assert_eq!(embedded_thumbnail(&without), None, "a metafile is not a picture made here");
        assert_eq!(embedded_thumbnail(b"%PDF-1.7"), None);
        assert_eq!(embedded_thumbnail(b"PK\x03\x04 not really a zip"), None);
    }
}
