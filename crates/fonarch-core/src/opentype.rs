use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// OpenType CFF (`OTTO`) or TrueType (`00 01 00 00`) magic.
pub fn magic_ext(path: &Path) -> Option<&'static str> {
    let mut file = File::open(path).ok()?;
    let mut buf = [0u8; 4];
    file.read_exact(&mut buf).ok()?;
    match &buf {
        b"OTTO" => Some(".otf"),
        [0x00, 0x01, 0x00, 0x00] => Some(".ttf"),
        _ => None,
    }
}

pub struct FallbackName {
    pub family: String,
    pub variation: String,
    pub full_name: String,
    pub is_variable: bool,
}

/// OpenType name table + `fvar` when entitlements.xml has no row for this id.
pub fn fallback_names(path: &Path) -> Option<FallbackName> {
    let mut file = File::open(path).ok()?;
    let len = file.seek(SeekFrom::End(0)).ok()? as usize;
    if !(16..=64 * 1024 * 1024).contains(&len) {
        return None;
    }
    file.seek(SeekFrom::Start(0)).ok()?;
    let mut data = Vec::with_capacity(len);
    file.read_to_end(&mut data).ok()?;
    let face = ttf_parser::Face::parse(&data, 0).ok()?;

    let family = best_name(&face, &[16, 1]).unwrap_or_else(|| "Unknown".into());
    let variation = best_name(&face, &[17, 2]).unwrap_or_else(|| "Regular".into());
    let full_name = best_name(&face, &[4]).unwrap_or_else(|| {
        if variation.is_empty() {
            family.clone()
        } else {
            format!("{family} {variation}")
        }
    });
    let is_variable = !face.variation_axes().is_empty();

    Some(FallbackName {
        family,
        variation,
        full_name,
        is_variable,
    })
}

fn best_name(face: &ttf_parser::Face<'_>, ids: &[u16]) -> Option<String> {
    for id in ids {
        let mut unicode = None;
        let mut any = None;
        for name in face.names() {
            if name.name_id != *id {
                continue;
            }
            let Some(text) = name.to_string() else {
                continue;
            };
            let text = text.trim();
            if text.is_empty() {
                continue;
            }
            if name.is_unicode() {
                unicode = Some(text.to_string());
                break;
            }
            if any.is_none() {
                any = Some(text.to_string());
            }
        }
        if unicode.is_some() {
            return unicode;
        }
        if any.is_some() {
            return any;
        }
    }
    None
}

/// Numeric livetype id from `.169.otf`, `169.otf`, or Windows `10294` (no extension).
pub fn id_from_filename(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_str()?;
    if name.eq_ignore_ascii_case(".ds_store") {
        return None;
    }
    let stem = [".otf", ".ttf", ".OTF", ".TTF"]
        .iter()
        .find_map(|ext| name.strip_suffix(ext))
        .unwrap_or(name);
    let id = stem.trim_start_matches('.');
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn magic_otto_and_ttf() {
        let dir = tempdir().unwrap();
        let otf = dir.path().join("a.otf");
        let ttf = dir.path().join("b.ttf");
        let junk = dir.path().join("c.bin");
        std::fs::write(&otf, b"OTTO\0\0\0\0").unwrap();
        std::fs::write(&ttf, [0x00, 0x01, 0x00, 0x00, 0, 0, 0, 0]).unwrap();
        std::fs::write(&junk, b"PK\x03\x04").unwrap();
        assert_eq!(magic_ext(&otf), Some(".otf"));
        assert_eq!(magic_ext(&ttf), Some(".ttf"));
        assert_eq!(magic_ext(&junk), None);
    }

    #[test]
    fn ids_from_hidden_and_plain() {
        assert_eq!(
            id_from_filename(Path::new("/tmp/.r/.169.otf")).as_deref(),
            Some("169")
        );
        assert_eq!(
            id_from_filename(Path::new("C:/livetype/.r/169.otf")).as_deref(),
            Some("169")
        );
        assert_eq!(id_from_filename(Path::new("/tmp/.DS_Store")), None);
        assert_eq!(
            id_from_filename(Path::new("C:/livetype/r/10294")).as_deref(),
            Some("10294")
        );
        // Extensionless digits are ids. Encrypted blobs live in e/ and are skipped there.
        assert_eq!(
            id_from_filename(Path::new("/tmp/.e/.169")).as_deref(),
            Some("169")
        );
    }

    #[test]
    fn encrypted_blob_is_not_ot() {
        let dir = tempdir().unwrap();
        let p = dir.path().join(".169");
        let mut f = File::create(&p).unwrap();
        f.write_all(&[0x9c, 0x7b, 0xd7, 0xb5, 0, 0, 0, 0]).unwrap();
        assert_eq!(magic_ext(&p), None);
    }
}
