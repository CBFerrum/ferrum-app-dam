//! What a file says of itself beyond EXIF: its XMP packet (in photos, PDFs and office documents
//! alike) and a JPEG's IPTC record (Photoshop's APP13 segment). Its title, caption, creator,
//! credit line, copyright and keywords; the place it shows (IPTC's location shown, never the
//! location created, where it was taken); the people it shows; and its licence's usage terms and
//! web address. Read as text and never run. XMP is read before IPTC, as the newer of the two; each
//! is only looked for where it is written.

/// What a file's XMP or IPTC says.
#[derive(Debug, Default, PartialEq)]
pub(crate) struct Said {
    pub title: Option<String>,
    pub caption: Option<String>,
    pub creator: Option<String>,
    pub credit: Option<String>,
    pub copyright: Option<String>,
    pub keywords: Vec<String>,
    pub place: Option<String>,
    pub people: Vec<String>,
    pub terms: Option<String>,
    pub licence: Option<String>,
}

/// A value kept: at most this many characters, and this many keywords.
const LONGEST: usize = 2000;
const KEYWORDS: usize = 50;

/// What `bytes` say of themselves: XMP first, then IPTC for what XMP leaves unsaid.
pub(crate) fn said(bytes: &[u8]) -> Said {
    let x = xmp(bytes).map(|p| from_xmp(&p)).unwrap_or_default();
    let i = iptc(bytes);
    Said {
        title: x.title.or(i.title),
        caption: x.caption.or(i.caption),
        creator: x.creator.or(i.creator),
        credit: x.credit.or(i.credit),
        copyright: x.copyright.or(i.copyright),
        keywords: if x.keywords.is_empty() { i.keywords } else { x.keywords },
        place: x.place.or(i.place),
        people: x.people,
        terms: x.terms,
        licence: x.licence,
    }
}

/// A place from its parts, the smallest first, those said joined: "Pier 4, Sydney, NSW, Australia".
fn place(parts: [Option<String>; 4]) -> Option<String> {
    let said: Vec<String> = parts.into_iter().flatten().collect();
    (!said.is_empty()).then(|| said.join(", "))
}

/// The inside of element `name`'s first occurrence (`<name …>…</name>`), if it has one.
fn inside<'a>(xmp: &'a str, name: &str) -> Option<&'a str> {
    let start = xmp.find(&format!("<{name}>")).or_else(|| xmp.find(&format!("<{name} ")))?;
    let open_end = start + xmp[start..].find('>')? + 1;
    let close = open_end + xmp[open_end..].find(&format!("</{name}>"))?;
    Some(&xmp[open_end..close])
}

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

/// The XMP packet in `bytes` (`<x:xmpmeta …>…</x:xmpmeta>`), if there is one in UTF-8.
fn xmp(bytes: &[u8]) -> Option<String> {
    let start = find(bytes, b"<x:xmpmeta")?;
    let end = start + find(&bytes[start..], b"</x:xmpmeta>")? + "</x:xmpmeta>".len();
    String::from_utf8(bytes[start..end].to_vec()).ok()
}

/// Text with XML's escapes undone, trimmed, at most [`LONGEST`] characters; nothing when empty.
fn clean(text: &str) -> Option<String> {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        let tail = &rest[i..];
        let Some(semi) = tail.find(';').filter(|s| *s <= 10) else {
            out.push('&');
            rest = &tail[1..];
            continue;
        };
        let entity = &tail[1..semi];
        let ch = match entity {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            e if e.starts_with("#x") || e.starts_with("#X") => u32::from_str_radix(&e[2..], 16).ok().and_then(char::from_u32),
            e if e.starts_with('#') => e[1..].parse().ok().and_then(char::from_u32),
            _ => None,
        };
        match ch {
            Some(c) => {
                out.push(c);
                rest = &tail[semi + 1..];
            }
            None => {
                out.push('&');
                rest = &tail[1..];
            }
        }
    }
    out.push_str(rest);
    let t: String = out.trim().chars().filter(|c| !c.is_control() || *c == '\n').take(LONGEST).collect();
    (!t.is_empty()).then_some(t)
}

/// The items of property `name` (`dc:subject`): its list's items, its own text, or an attribute of
/// that name. For a language alternative, the default language's item comes first.
fn values(xmp: &str, name: &str) -> Vec<String> {
    let open = format!("<{name}");
    let mut from = 0;
    while let Some(i) = xmp[from..].find(&open).map(|i| i + from) {
        let after = &xmp[i + open.len()..];
        from = i + open.len();
        // Not a longer name that starts the same.
        if !after.starts_with(|c: char| c == '>' || c.is_whitespace() || c == '/') {
            continue;
        }
        let Some(gt) = after.find('>') else { break };
        if after[..gt].ends_with('/') {
            continue;
        }
        let close = format!("</{name}>");
        let Some(end) = after[gt + 1..].find(&close) else { break };
        let body = &after[gt + 1..gt + 1 + end];
        let mut items: Vec<(bool, String)> = Vec::new();
        let mut at = 0;
        while let Some(li) = body[at..].find("<rdf:li").map(|l| l + at) {
            let tag_end = match body[li..].find('>') {
                Some(t) => li + t,
                None => break,
            };
            let tag = &body[li..tag_end];
            let Some(stop) = body[tag_end + 1..].find("</rdf:li>").map(|s| s + tag_end + 1) else { break };
            if let Some(text) = clean(&body[tag_end + 1..stop]) {
                items.push((tag.contains("x-default"), text));
            }
            at = stop + "</rdf:li>".len();
        }
        if items.is_empty() {
            return clean(body).into_iter().collect();
        }
        // The default language's first.
        items.sort_by_key(|(default, _)| !*default);
        return items.into_iter().map(|(_, t)| t).collect();
    }
    // An attribute of an rdf:Description: `name="…"` or `name='…'`.
    for quote in ['"', '\''] {
        let attr = format!("{name}={quote}");
        if let Some(i) = xmp.find(&attr) {
            let rest = &xmp[i + attr.len()..];
            if let Some(end) = rest.find(quote) {
                return clean(&rest[..end]).into_iter().collect();
            }
        }
    }
    vec![]
}

fn from_xmp(xmp: &str) -> Said {
    let first = |name: &str| values(xmp, name).into_iter().next();
    let creators = values(xmp, "dc:creator");
    // The location shown, from IPTC Extension's structure or else IPTC Core's fields, which name
    // the place shown too. Never the location created.
    let shown = inside(xmp, "Iptc4xmpExt:LocationShown").map(|block| {
        let part = |name: &str| values(block, name).into_iter().next();
        place([part("Iptc4xmpExt:Sublocation"), part("Iptc4xmpExt:City"), part("Iptc4xmpExt:ProvinceState"), part("Iptc4xmpExt:CountryName")])
    });
    let core = || place([first("Iptc4xmpCore:Location"), first("photoshop:City"), first("photoshop:State"), first("photoshop:Country")]);
    let licence = first("xmpRights:WebStatement").filter(|u| u.starts_with("https://") || u.starts_with("http://"));
    Said {
        title: first("dc:title"),
        caption: first("dc:description"),
        creator: (!creators.is_empty()).then(|| creators.join(", ")),
        credit: first("photoshop:Credit"),
        copyright: first("dc:rights"),
        keywords: values(xmp, "dc:subject").into_iter().take(KEYWORDS).collect(),
        place: shown.flatten().or_else(core),
        people: values(xmp, "Iptc4xmpExt:PersonInImage").into_iter().take(KEYWORDS).collect(),
        terms: first("xmpRights:UsageTerms"),
        licence,
    }
}

/// A JPEG's IPTC record, in its Photoshop (APP13) segment's IPTC-NAA resource.
fn iptc(bytes: &[u8]) -> Said {
    if !bytes.starts_with(&[0xFF, 0xD8]) {
        return Said::default();
    }
    let mut i = 2;
    while i + 4 <= bytes.len() && bytes[i] == 0xFF {
        let marker = bytes[i + 1];
        // The image data, or its end: no metadata after.
        if marker == 0xDA || marker == 0xD9 {
            break;
        }
        if marker == 0x01 || (0xD0..=0xD7).contains(&marker) {
            i += 2;
            continue;
        }
        let len = usize::from(u16::from_be_bytes([bytes[i + 2], bytes[i + 3]]));
        if len < 2 || i + 2 + len > bytes.len() {
            break;
        }
        let segment = &bytes[i + 4..i + 2 + len];
        if marker == 0xED
            && let Some(resources) = segment.strip_prefix(b"Photoshop 3.0\0")
        {
            return from_resources(resources);
        }
        i += 2 + len;
    }
    Said::default()
}

/// Photoshop's resources (`8BIM`, its id, a padded name, a size, the data padded to even), the
/// IPTC-NAA one (0x0404) read.
fn from_resources(mut d: &[u8]) -> Said {
    while d.len() >= 12 && d.starts_with(b"8BIM") {
        let id = u16::from_be_bytes([d[4], d[5]]);
        let name = 1 + usize::from(d[6]);
        let at = 6 + name + name % 2;
        let Some(size) = d.get(at..at + 4).map(|s| u32::from_be_bytes([s[0], s[1], s[2], s[3]]) as usize) else { break };
        let start = at + 4;
        let Some(data) = d.get(start..start + size) else { break };
        if id == 0x0404 {
            return from_iim(data);
        }
        d = d.get(start + size + size % 2..).unwrap_or_default();
    }
    Said::default()
}

/// IPTC's datasets of record 2: the title (5), keywords (25), the by-line (80), the city (90), the
/// sub-location (92), the province or state (95), the country (101), the credit (110), the
/// copyright (116) and the caption (120). Text is UTF-8, or else Latin-1.
fn from_iim(mut d: &[u8]) -> Said {
    let mut s = Said::default();
    let (mut city, mut sub, mut province, mut country) = (None, None, None, None);
    while d.len() >= 5 && d[0] == 0x1C {
        let (record, set) = (d[1], d[2]);
        let len = u16::from_be_bytes([d[3], d[4]]);
        // An extended dataset is not text kept here.
        if len & 0x8000 != 0 {
            break;
        }
        let Some(raw) = d.get(5..5 + usize::from(len)) else { break };
        let text = match std::str::from_utf8(raw) {
            Ok(t) => t.to_string(),
            Err(_) => raw.iter().map(|&b| char::from(b)).collect(),
        };
        if record == 2
            && let Some(t) = clean_text(&text)
        {
            // The first of each, and the keywords in order.
            let first = |field: &mut Option<String>, t: String| {
                field.get_or_insert(t);
            };
            match set {
                5 => first(&mut s.title, t),
                25 if s.keywords.len() < KEYWORDS => s.keywords.push(t),
                80 => first(&mut s.creator, t),
                90 => first(&mut city, t),
                92 => first(&mut sub, t),
                95 => first(&mut province, t),
                101 => first(&mut country, t),
                110 => first(&mut s.credit, t),
                116 => first(&mut s.copyright, t),
                120 => first(&mut s.caption, t),
                _ => {}
            }
        }
        d = &d[5 + usize::from(len)..];
    }
    s.place = place([sub, city, province, country]);
    s
}

/// Plain text trimmed, without control characters, at most [`LONGEST`] characters.
fn clean_text(text: &str) -> Option<String> {
    let t: String = text.trim().chars().filter(|c| !c.is_control() || *c == '\n').take(LONGEST).collect();
    (!t.is_empty()).then_some(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PACKET: &str = r#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?><x:xmpmeta xmlns:x="adobe:ns:meta/"><rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
<rdf:Description rdf:about="" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:photoshop="http://ns.adobe.com/photoshop/1.0/" photoshop:Credit="Northwind &amp; Co">
<dc:title><rdf:Alt><rdf:li xml:lang="fr">Lancement</rdf:li><rdf:li xml:lang="x-default">Spring launch</rdf:li></rdf:Alt></dc:title>
<dc:description><rdf:Alt><rdf:li xml:lang="x-default">Sage, sand &amp; slate &#8211; the new range</rdf:li></rdf:Alt></dc:description>
<dc:creator><rdf:Seq><rdf:li>Jo Doe</rdf:li><rdf:li>Sam Lee</rdf:li></rdf:Seq></dc:creator>
<dc:creatorTool>Not the creator</dc:creatorTool>
<dc:rights><rdf:Alt><rdf:li xml:lang="x-default">© 2026 Northwind</rdf:li></rdf:Alt></dc:rights>
<dc:subject><rdf:Bag><rdf:li>spring</rdf:li><rdf:li>launch</rdf:li></rdf:Bag></dc:subject>
<Iptc4xmpExt:LocationCreated><rdf:Bag><rdf:li rdf:parseType="Resource"><Iptc4xmpExt:City>Never kept</Iptc4xmpExt:City></rdf:li></rdf:Bag></Iptc4xmpExt:LocationCreated>
<Iptc4xmpExt:LocationShown><rdf:Bag><rdf:li rdf:parseType="Resource"><Iptc4xmpExt:Sublocation>Pier 4</Iptc4xmpExt:Sublocation><Iptc4xmpExt:City>Sydney</Iptc4xmpExt:City><Iptc4xmpExt:CountryName>Australia</Iptc4xmpExt:CountryName></rdf:li></rdf:Bag></Iptc4xmpExt:LocationShown>
<Iptc4xmpExt:PersonInImage><rdf:Bag><rdf:li>Ana Rao</rdf:li><rdf:li>Bo Lin</rdf:li></rdf:Bag></Iptc4xmpExt:PersonInImage>
<xmpRights:UsageTerms><rdf:Alt><rdf:li xml:lang="x-default">Editorial use only, credit Northwind</rdf:li></rdf:Alt></xmpRights:UsageTerms>
<xmpRights:WebStatement>https://northwind.test/licence</xmpRights:WebStatement>
</rdf:Description></rdf:RDF></x:xmpmeta><?xpacket end="w"?>"#;

    /// A JPEG holding an IPTC record of `datasets` (dataset, text) in its APP13 segment.
    fn jpeg_with_iptc(datasets: &[(u8, &[u8])]) -> Vec<u8> {
        let mut iim = vec![0x1C, 1, 90, 0, 3, 0x1B, 0x25, 0x47];
        for (set, text) in datasets {
            iim.extend_from_slice(&[0x1C, 2, *set]);
            iim.extend_from_slice(&u16::try_from(text.len()).unwrap().to_be_bytes());
            iim.extend_from_slice(text);
        }
        let mut resource = b"8BIM\x04\x04\x00\x00".to_vec();
        resource.extend_from_slice(&u32::try_from(iim.len()).unwrap().to_be_bytes());
        resource.extend_from_slice(&iim);
        if iim.len() % 2 == 1 {
            resource.push(0);
        }
        let mut segment = b"Photoshop 3.0\0".to_vec();
        segment.extend_from_slice(&resource);
        let mut out = vec![0xFF, 0xD8, 0xFF, 0xED];
        out.extend_from_slice(&u16::try_from(segment.len() + 2).unwrap().to_be_bytes());
        out.extend_from_slice(&segment);
        out.extend_from_slice(&[0xFF, 0xD9]);
        out
    }

    #[test]
    fn xmp_in_any_file() {
        let mut pdf = b"%PDF-1.7\n1 0 obj << /Type /Metadata /Subtype /XML >> stream\n".to_vec();
        pdf.extend_from_slice(PACKET.as_bytes());
        pdf.extend_from_slice(b"\nendstream endobj\n%%EOF\n");
        let s = said(&pdf);
        assert_eq!(s.title.as_deref(), Some("Spring launch"), "the default language's");
        assert_eq!(s.caption.as_deref(), Some("Sage, sand & slate \u{2013} the new range"), "escapes undone");
        assert_eq!(s.creator.as_deref(), Some("Jo Doe, Sam Lee"), "not dc:creatorTool");
        assert_eq!((s.credit.as_deref(), s.copyright.as_deref()), (Some("Northwind & Co"), Some("© 2026 Northwind")));
        assert_eq!(s.keywords, ["spring", "launch"]);
        assert_eq!(s.place.as_deref(), Some("Pier 4, Sydney, Australia"), "the location shown, never the location created");
        assert_eq!(s.people, ["Ana Rao", "Bo Lin"]);
        assert_eq!((s.terms.as_deref(), s.licence.as_deref()), (Some("Editorial use only, credit Northwind"), Some("https://northwind.test/licence")));
        let not_web = PACKET.replace("https://northwind.test/licence", "javascript:alert(1)");
        assert_eq!(said(not_web.as_bytes()).licence, None, "only a web address is kept");
        assert_eq!(said(b"%PDF-1.7 nothing said"), Said::default());
        assert_eq!(said(b"<x:xmpmeta never closed"), Said::default());
    }

    #[test]
    fn iptc_in_a_jpeg() {
        let jpeg = jpeg_with_iptc(&[
            (5, b"Harbour at dawn"),
            (80, b"Jo Doe"),
            (116, b"Brand Co"),
            (25, b"harbour"),
            (25, b"Caf\xe9"),
            (120, b"Boats  "),
            (90, b"Hobart"),
            (101, b"Australia"),
        ]);
        let s = said(&jpeg);
        assert_eq!(
            (s.title.as_deref(), s.creator.as_deref(), s.copyright.as_deref(), s.caption.as_deref()),
            (Some("Harbour at dawn"), Some("Jo Doe"), Some("Brand Co"), Some("Boats"))
        );
        assert_eq!(s.keywords, ["harbour", "Café"], "Latin-1 read as such");
        assert_eq!(s.place.as_deref(), Some("Hobart, Australia"));
        // XMP is read first; IPTC says what it leaves unsaid.
        let mut both = jpeg_with_iptc(&[(5, b"From IPTC"), (110, b"IPTC credit")]);
        both.truncate(both.len() - 2);
        both.extend_from_slice(PACKET.as_bytes());
        both.extend_from_slice(&[0xFF, 0xD9]);
        let s = said(&both);
        assert_eq!((s.title.as_deref(), s.credit.as_deref()), (Some("Spring launch"), Some("Northwind & Co")));
        // Truncated or odd segments say nothing rather than fail.
        assert_eq!(said(&jpeg[..20]), Said::default());
        assert_eq!(said(&[0xFF, 0xD8, 0xFF, 0xED, 0x00, 0x01]), Said::default());
    }
}
