use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use walkdir::{DirEntry, WalkDir};

use crate::error::{Error, Result};
use crate::opentype::magic_ext;

const SKIP_DIR_NAMES: &[&str] = &[
    "node_modules",
    ".git",
    "target",
    ".Trash",
    "Trash",
    "Caches",
    "Cache",
    "Logs",
    "Containers",
    "Group Containers",
    "Mail",
    "Messages",
    "Photos Library.photoslibrary",
    "FontBase",
    "GudeLivetype",
    "e",
    ".cache",
    "Movies",
    "Music",
    "Pictures",
    "Downloads",
    "Applications",
    "Library Daemon",
    "iTunes",
    "CloudStorage",
];

/// Known Creative Cloud cache: `~/Library/Application Support/Adobe/...` on
/// Mac, `%APPDATA%\Adobe\...` on Windows.
pub fn known_livetype_path() -> Option<PathBuf> {
    let mut p = dirs::data_dir()?;
    p.push("Adobe");
    p.push("CoreSync");
    p.push("plugins");
    p.push("livetype");
    Some(p)
}

const ENTITLEMENTS_DIRS: &[&str] = &[".c", "c"];
const FONT_BUCKETS: &[&str] = &[".r", "r", ".w", "w", ".t", "t"];

/// Mac: `livetype/.c/entitlements.xml`. Windows: `livetype/c/entitlements.xml`.
pub fn entitlements_path(livetype: &Path) -> PathBuf {
    for dir in ENTITLEMENTS_DIRS {
        let p = livetype.join(dir).join("entitlements.xml");
        if p.is_file() {
            return p;
        }
    }
    livetype.join(".c").join("entitlements.xml")
}

/// `livetype` + entitlements catalog + a font bucket with real OpenType.
/// Mac buckets are dotted (`.r`); Windows drops the dot (`r`).
pub fn is_livetype(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }
    if path.file_name().and_then(|n| n.to_str()) != Some("livetype") {
        return false;
    }
    let xml = entitlements_path(path);
    if !xml.is_file() {
        return false;
    }
    let Ok(head) = fs::read(&xml) else {
        return false;
    };
    if head.len() < 32 {
        return false;
    }
    let probe = String::from_utf8_lossy(&head[..head.len().min(400)]);
    if !probe.contains("<typekitSyncState") {
        return false;
    }
    FONT_BUCKETS
        .iter()
        .any(|dir| bucket_has_opentype(&path.join(dir)))
}

fn bucket_has_opentype(dir: &Path) -> bool {
    let Ok(rd) = fs::read_dir(dir) else {
        return false;
    };
    rd.filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .take(24)
        .any(|e| magic_ext(&e.path()).is_some())
}

fn skip_dir(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    if SKIP_DIR_NAMES.iter().any(|s| name.eq_ignore_ascii_case(s)) {
        return true;
    }
    if name.starts_with('.') && !matches!(name.as_ref(), ".c" | ".r" | ".w" | ".t") {
        return true;
    }
    false
}

fn collect_livetype_under(root: &Path, max_depth: usize, out: &mut Vec<PathBuf>) {
    let walker = WalkDir::new(root)
        .follow_links(false)
        .max_depth(max_depth)
        .into_iter()
        .filter_entry(|e| !skip_dir(e));
    for entry in walker.flatten() {
        if entry.file_type().is_dir()
            && entry.file_name() == "livetype"
            && is_livetype(entry.path())
        {
            out.push(entry.path().to_path_buf());
        }
    }
}

fn adobe_shaped_under_home(home: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let walker = WalkDir::new(home)
        .follow_links(false)
        .max_depth(8)
        .into_iter()
        .filter_entry(|e| !skip_dir(e));
    for entry in walker.flatten() {
        if entry.file_type().is_dir() && entry.file_name() == "Adobe" {
            collect_livetype_under(entry.path(), 8, &mut found);
        }
    }
    found
}

fn newest(candidates: &[PathBuf]) -> Option<PathBuf> {
    candidates
        .iter()
        .max_by_key(|p| {
            fs::metadata(entitlements_path(p))
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH)
        })
        .cloned()
}

pub struct Hunt {
    pub livetype: PathBuf,
    pub via: &'static str,
}

/// Known path, then Adobe-shaped trees under home. Does not scan the whole
/// volume — that waits behind a status line if these miss.
pub fn find_livetype() -> Result<Hunt> {
    if let Some(known) = known_livetype_path() {
        if is_livetype(&known) {
            return Ok(Hunt {
                livetype: known,
                via: "known path",
            });
        }
    }

    let Some(home) = dirs::home_dir() else {
        return Err(Error::NotFound);
    };

    let shaped = adobe_shaped_under_home(&home);
    if let Some(p) = newest(&shaped) {
        return Ok(Hunt {
            livetype: p,
            via: "Adobe-shaped path",
        });
    }

    let mut named = Vec::new();
    collect_livetype_under(&home, 10, &mut named);
    if let Some(p) = newest(&named) {
        return Ok(Hunt {
            livetype: p,
            via: "home search",
        });
    }

    Err(Error::NotFound)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write_fake_livetype(root: &Path) -> PathBuf {
        let lt = root.join("Adobe/CoreSync/plugins/livetype");
        fs::create_dir_all(lt.join(".c")).unwrap();
        fs::create_dir_all(lt.join(".r")).unwrap();
        fs::write(
            lt.join(".c/entitlements.xml"),
            b"<?xml version=\"1.0\"?><typekitSyncState><fonts/></typekitSyncState>",
        )
        .unwrap();
        fs::write(lt.join(".r/.1.otf"), b"OTTO\0\0\0\0").unwrap();
        lt
    }

    #[test]
    fn fingerprint_accepts_real_shape() {
        let dir = tempdir().unwrap();
        let lt = write_fake_livetype(dir.path());
        assert!(is_livetype(&lt));
    }

    #[test]
    fn fingerprint_rejects_missing_xml_or_fonts() {
        let dir = tempdir().unwrap();
        let lt = dir.path().join("livetype");
        fs::create_dir_all(&lt).unwrap();
        assert!(!is_livetype(&lt));
        fs::create_dir_all(lt.join(".c")).unwrap();
        fs::write(lt.join(".c/entitlements.xml"), b"<html/>").unwrap();
        assert!(!is_livetype(&lt));
    }

    #[test]
    fn gude_is_not_livetype() {
        let dir = tempdir().unwrap();
        let gude = dir.path().join("GudeLivetype");
        fs::create_dir_all(&gude).unwrap();
        assert!(!is_livetype(&gude));
    }

    fn write_windows_livetype(root: &Path) -> PathBuf {
        let lt = root.join("Adobe/CoreSync/plugins/livetype");
        fs::create_dir_all(lt.join("c")).unwrap();
        fs::create_dir_all(lt.join("r")).unwrap();
        fs::create_dir_all(lt.join("e")).unwrap();
        fs::write(
            lt.join("c/entitlements.xml"),
            b"<?xml version=\"1.0\"?><typekitSyncState><fonts/></typekitSyncState>",
        )
        .unwrap();
        fs::write(lt.join("r/10294"), b"OTTO\0\0\0\0").unwrap();
        fs::write(lt.join("e/10294"), [0x9c, 0x7b, 0xd7, 0xb5, 0, 0, 0, 0]).unwrap();
        lt
    }

    #[test]
    fn fingerprint_accepts_windows_shape() {
        let dir = tempdir().unwrap();
        let lt = write_windows_livetype(dir.path());
        assert!(is_livetype(&lt));
        assert_eq!(entitlements_path(&lt), lt.join("c/entitlements.xml"));
    }

    #[test]
    fn windows_e_only_is_not_livetype() {
        let dir = tempdir().unwrap();
        let lt = dir.path().join("livetype");
        fs::create_dir_all(lt.join("c")).unwrap();
        fs::create_dir_all(lt.join("e")).unwrap();
        fs::write(
            lt.join("c/entitlements.xml"),
            b"<?xml version=\"1.0\"?><typekitSyncState><fonts/></typekitSyncState>",
        )
        .unwrap();
        fs::write(lt.join("e/1"), b"OTTO\0\0\0\0").unwrap();
        assert!(!is_livetype(&lt));
    }
}
