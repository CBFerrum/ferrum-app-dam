//! A watermark across a picture a shared link hands out: its words, white with a dark edge and
//! partly see-through, repeated on a slant over the whole picture. The words are set in Helvetica,
//! the standard PDF font hayro carries (PDFium's), on a page the picture's size, and drawn by hayro
//! on a clear background before they are laid over the picture. Letters outside Latin-1 show as
//! `?`, as Helvetica has no others. The marked copy carries none of the file's EXIF: it is turned
//! the way the camera was held, and where it was taken goes no further.

use image::{DynamicImage, ImageFormat, ImageReader, Limits, RgbaImage};
use std::io::Cursor;

/// Pictures larger than this are not marked (as renditions are not made of them).
const MAX_PIXELS: u64 = 40_000_000;

/// The picture in `bytes` with `words` across it: as JPEG, or as PNG when it has transparency.
pub(crate) fn watermarked(bytes: &[u8], words: &str) -> Result<(Vec<u8>, &'static str), String> {
    let mut reader = ImageReader::new(Cursor::new(bytes)).with_guessed_format().map_err(|e| format!("could not be read: {e}"))?;
    let mut limits = Limits::default();
    limits.max_alloc = Some(512 << 20);
    reader.limits(limits);
    let mut picture = reader.decode().map_err(|e| format!("could not be read as a picture: {e}"))?;
    // Turned the way the camera was held first: the marked copy carries no EXIF to say so.
    if let Some(o) = crate::renditions::facts(bytes).orientation.and_then(image::metadata::Orientation::from_exif) {
        picture.apply_orientation(o);
    }
    let (w, h) = (picture.width(), picture.height());
    if u64::from(w) * u64::from(h) > MAX_PIXELS || w > u32::from(u16::MAX) || h > u32::from(u16::MAX) {
        return Err(format!("is {w} by {h} pixels, more than a watermark is drawn on"));
    }
    let alpha = picture.color().has_alpha();
    let mut marked = picture.to_rgba8();
    image::imageops::overlay(&mut marked, &words_layer(w, h, words)?, 0, 0);
    let mut out = Vec::new();
    let written = if alpha {
        DynamicImage::ImageRgba8(marked).write_to(Cursor::new(&mut out), ImageFormat::Png).map(|_| "image/png")
    } else {
        DynamicImage::ImageRgb8(DynamicImage::ImageRgba8(marked).to_rgb8())
            .write_with_encoder(image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 88))
            .map(|_| "image/jpeg")
    };
    written.map(|ctype| (out, ctype)).map_err(|e| format!("could not be written: {e}"))
}

/// `words` as a PDF string's bytes: Latin-1, each byte past ASCII and each `\`, `(` and `)` escaped.
fn pdf_string(words: &str) -> String {
    let mut out = String::from("(");
    for c in words.chars() {
        let b = u32::from(c);
        match c {
            '\\' | '(' | ')' => {
                out.push('\\');
                out.push(c);
            }
            ' '..='~' => out.push(c),
            _ if (0xA0..=0xFF).contains(&b) => out.push_str(&format!("\\{b:03o}")),
            _ => out.push('?'),
        }
    }
    out.push(')');
    out
}

/// The words alone, repeated on a slant across a clear picture `w` by `h`.
fn words_layer(w: u32, h: u32, words: &str) -> Result<RgbaImage, String> {
    let words = words.trim();
    let size = (w.min(h) as f32 / 14.0).clamp(14.0, 160.0);
    let text = pdf_string(words);
    // Helvetica's letters average about half their size across.
    let across = size * (0.55 * words.chars().count() as f32 + 3.0);
    let down = size * 3.0;
    let (sin, cos) = 30f32.to_radians().sin_cos();
    let mut content = format!("q /Marked gs 2 Tr 1 1 1 rg 0.15 0.15 0.15 RG {:.2} w BT /Words {size:.1} Tf\n", size / 24.0);
    let (mut y, mut row) = (-(h as f32), 0);
    while y < 2.0 * h as f32 {
        // Every other row stands half a step along, so the words do not line up in columns.
        let mut x = -(w as f32) + if row % 2 == 1 { across / 2.0 } else { 0.0 };
        while x < 2.0 * w as f32 {
            content.push_str(&format!("{cos:.4} {sin:.4} {:.4} {cos:.4} {x:.1} {y:.1} Tm {text} Tj\n", -sin));
            x += across;
        }
        y += down;
        row += 1;
    }
    content.push_str("ET Q");
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
        format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {w} {h}] /Resources << /Font << /Words 5 0 R >> /ExtGState << /Marked 6 0 R >> >> /Contents 4 0 R >>"
        ),
        format!("<< /Length {} >>\nstream\n{content}\nendstream", content.len()),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>".to_string(),
        "<< /Type /ExtGState /ca 0.4 /CA 0.4 >>".to_string(),
    ];
    let mut pdf = b"%PDF-1.4\n".to_vec();
    let mut offsets = vec![];
    for (i, o) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.extend(format!("{} 0 obj\n{o}\nendobj\n", i + 1).bytes());
    }
    let xref = pdf.len();
    pdf.extend(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).bytes());
    for o in offsets {
        pdf.extend(format!("{o:010} 00000 n \n").bytes());
    }
    pdf.extend(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objects.len() + 1).bytes());

    let doc = hayro::hayro_syntax::Pdf::new(pdf).map_err(|_| "its watermark could not be set".to_string())?;
    let page = doc.pages().first().ok_or("its watermark could not be set")?;
    let settings = hayro::PixmapSettings { x_scale: 1.0, y_scale: 1.0, bg_color: hayro::vello_cpu::color::palette::css::TRANSPARENT };
    let layer = hayro::render(page, &hayro::RenderCache::new(), &Default::default(), &hayro::RenderSettings::default(), &settings);
    let png = layer.into_png().map_err(|e| format!("its watermark could not be drawn: {e}"))?;
    let layer = image::load_from_memory(&png).map_err(|e| format!("its watermark could not be drawn: {e}"))?.to_rgba8();
    Ok(if layer.dimensions() == (w, h) { layer } else { image::imageops::resize(&layer, w, h, image::imageops::FilterType::Triangle) })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn picture(w: u32, h: u32, alpha: bool) -> Vec<u8> {
        let mut out = Vec::new();
        let img = if alpha {
            DynamicImage::ImageRgba8(RgbaImage::from_pixel(w, h, image::Rgba([90, 130, 170, 200])))
        } else {
            DynamicImage::ImageRgb8(image::RgbImage::from_pixel(w, h, image::Rgb([90, 130, 170])))
        };
        img.write_to(Cursor::new(&mut out), if alpha { ImageFormat::Png } else { ImageFormat::Jpeg }).unwrap();
        out
    }

    #[test]
    fn words_drawn_across_a_picture() {
        let (marked, ctype) = watermarked(&picture(400, 300, false), "PREVIEW").unwrap();
        assert_eq!(ctype, "image/jpeg");
        let img = image::load_from_memory(&marked).unwrap().to_rgb8();
        assert_eq!(img.dimensions(), (400, 300));
        // The letters, white with a dark edge, change the blue in every quarter of the picture.
        let changed = |x0: u32, y0: u32| {
            (x0..x0 + 200)
                .flat_map(|x| (y0..y0 + 150).map(move |y| (x, y)))
                .filter(|&(x, y)| {
                    let p = img.get_pixel(x, y).0;
                    (i32::from(p[0]) - 90).abs() > 30 || (i32::from(p[2]) - 170).abs() > 30
                })
                .count()
        };
        for (x0, y0) in [(0, 0), (200, 0), (0, 150), (200, 150)] {
            let n = changed(x0, y0);
            assert!(n > 200 * 150 / 100, "{n} pixels changed in the quarter at ({x0}, {y0})");
        }
        let (_, ctype) = watermarked(&picture(120, 80, true), "PREVIEW").unwrap();
        assert_eq!(ctype, "image/png", "transparency kept");
    }

    #[test]
    fn words_written_for_helvetica() {
        assert_eq!(pdf_string("Brand (Co) \\ é"), "(Brand \\(Co\\) \\\\ \\351)");
        assert_eq!(pdf_string("Привет"), "(??????)");
        assert!(watermarked(b"not a picture", "PREVIEW").unwrap_err().starts_with("could not be read"));
    }
}
