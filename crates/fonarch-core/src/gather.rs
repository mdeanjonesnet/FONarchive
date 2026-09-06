use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufWriter, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::Local;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::catalog::{parse_entitlements, FontMeta};
use crate::error::{Error, Result};
use crate::hunt::{entitlements_path, find_livetype};
use crate::names::dest_parts;
use crate::opentype::{fallback_names, id_from_filename, magic_ext};

const SOURCE_BUCKETS: &[(&str, u8)] = &[(".r", 0), (".w", 1), (".t", 2)];

#[derive(Debug, Clone, Default)]
pub struct GatherOptions {
    /// Parent of the dated `FONarch YYYY-MM-DD` folder. Default: Desktop.
    pub dest_parent: Option<PathBuf>,
    pub dry_run: bool,
    /// Skip hunt and use this livetype root (tests).
    pub livetype: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub enum GatherEvent {
    Status(String),
    Found {
        path: PathBuf,
        via: String,
    },
    Plan {
        total: usize,
        families: usize,
    },
    Font {
        name: String,
        index: usize,
        total: usize,
    },
    Warn(String),
    Zip {
        name: String,
        index: usize,
        total: usize,
    },
    Done {
        report: GatherReport,
    },
}

#[derive(Debug, Clone)]
pub struct GatherReport {
    pub output: PathBuf,
    pub copied: usize,
    pub families: usize,
    pub skipped: usize,
    pub fallback: usize,
    pub variable: usize,
    pub dry_run: bool,
    pub archived: bool,
}

struct Planned {
    id: String,
    src: PathBuf,
    family: String,
    variation: String,
    full_name: String,
    ext: &'static str,
    fallback: bool,
    is_variable: bool,
}

pub fn gather(opts: GatherOptions, on_event: impl FnMut(GatherEvent)) -> Result<GatherReport> {
    run(opts, on_event, false)
}

/// Same as GATHER, then zip the folder and delete it. `report.output` is the zip.
pub fn archive(opts: GatherOptions, on_event: impl FnMut(GatherEvent)) -> Result<GatherReport> {
    run(opts, on_event, true)
}

fn run(
    opts: GatherOptions,
    mut on_event: impl FnMut(GatherEvent),
    zip_after: bool,
) -> Result<GatherReport> {
    let hunt = if let Some(path) = opts.livetype.clone() {
        on_event(GatherEvent::Found {
            path: path.clone(),
            via: "override".into(),
        });
        path
    } else {
        on_event(GatherEvent::Status("Looking for livetype…".into()));
        let hit = find_livetype()?;
        on_event(GatherEvent::Found {
            path: hit.livetype.clone(),
            via: hit.via.into(),
        });
        hit.livetype
    };

    let xml = entitlements_path(&hunt);
    on_event(GatherEvent::Status(format!("Reading {}", xml.display())));
    let catalog = parse_entitlements(&xml)?;

    let planned = plan_copies(&hunt, &catalog, &mut on_event);
    let families: BTreeSet<&str> = planned.iter().map(|p| p.family.as_str()).collect();
    let total = planned.len();
    on_event(GatherEvent::Plan {
        total,
        families: families.len(),
    });

    let dest_parent = match opts.dest_parent {
        Some(p) => p,
        None => dirs::desktop_dir().ok_or(Error::NoDesktop)?,
    };
    let output = unique_output_dir(&dest_parent);

    if opts.dry_run {
        for (i, item) in planned.iter().enumerate() {
            on_event(GatherEvent::Font {
                name: item.full_name.clone(),
                index: i + 1,
                total,
            });
        }
        let mut report = GatherReport {
            output,
            copied: total,
            families: families.len(),
            skipped: 0,
            fallback: planned.iter().filter(|p| p.fallback).count(),
            variable: planned.iter().filter(|p| p.is_variable).count(),
            dry_run: true,
            archived: zip_after,
        };
        if zip_after {
            report.output = zip_path_for(&report.output);
        }
        on_event(GatherEvent::Done {
            report: report.clone(),
        });
        return Ok(report);
    }

    fs::create_dir_all(&output)?;

    let mut copied = 0usize;
    let mut skipped = 0usize;
    let mut used_names: BTreeMap<PathBuf, ()> = BTreeMap::new();
    let mut family_dirs: BTreeSet<String> = BTreeSet::new();

    for (i, item) in planned.iter().enumerate() {
        let (fam_dir, file) = dest_parts(&item.family, &item.variation, item.ext);
        let family_path = output.join(&fam_dir);
        if let Err(e) = fs::create_dir_all(&family_path) {
            on_event(GatherEvent::Warn(format!("skip {}: {e}", item.full_name)));
            skipped += 1;
            continue;
        }
        family_dirs.insert(fam_dir);
        let dest = unique_file(&family_path, &file, &mut used_names);
        match copy_font(&item.src, &dest) {
            Ok(_) => {
                copied += 1;
                on_event(GatherEvent::Font {
                    name: item.full_name.clone(),
                    index: i + 1,
                    total,
                });
            }
            Err(e) => {
                on_event(GatherEvent::Warn(format!(
                    "skip {} ({}): {e}",
                    item.full_name,
                    item.src.display()
                )));
                skipped += 1;
            }
        }
    }

    let mut report = GatherReport {
        families: family_dirs.len(),
        output,
        copied,
        skipped,
        fallback: planned.iter().filter(|p| p.fallback).count(),
        variable: planned.iter().filter(|p| p.is_variable).count(),
        dry_run: false,
        archived: false,
    };

    if zip_after {
        on_event(GatherEvent::Status("Zipping…".into()));
        let zip_path = zip_folder(&report.output, &mut on_event)?;
        // Finder writes .DS_Store into Desktop folders while we delete;
        // that is os error 66 (directory not empty) if we rmdir in place.
        remove_output_dir(&report.output)?;
        report.output = zip_path;
        report.archived = true;
    }

    on_event(GatherEvent::Done {
        report: report.clone(),
    });
    Ok(report)
}

/// Read/write copy so Desktop files get *now* timestamps and real bytes.
/// `fs::copy` on APFS clonefiles and keeps Adobe's mtimes, which makes a
/// second GATHER look like a renamed duplicate of the first.
fn copy_font(src: &Path, dest: &Path) -> std::io::Result<u64> {
    let mut from = std::io::BufReader::with_capacity(64 * 1024, fs::File::open(src)?);
    let mut to = std::io::BufWriter::with_capacity(64 * 1024, fs::File::create(dest)?);
    let n = std::io::copy(&mut from, &mut to)?;
    to.flush()?;
    Ok(n)
}

fn zip_path_for(dir: &Path) -> PathBuf {
    let mut p = dir.as_os_str().to_os_string();
    p.push(".zip");
    PathBuf::from(p)
}

fn unique_zip_path(dir: &Path) -> PathBuf {
    let primary = zip_path_for(dir);
    if !primary.exists() {
        return primary;
    }
    for i in 2..1000 {
        let mut p = dir.as_os_str().to_os_string();
        p.push(format!("_{i}.zip"));
        let p = PathBuf::from(p);
        if !p.exists() {
            return p;
        }
    }
    zip_path_for(dir)
}

fn zip_folder(dir: &Path, on_event: &mut impl FnMut(GatherEvent)) -> Result<PathBuf> {
    let zip_path = unique_zip_path(dir);
    let file = fs::File::create(&zip_path)?;
    let mut zip = ZipWriter::new(BufWriter::new(file));
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let files: Vec<_> = walkdir::WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .collect();
    let total = files.len();
    for (i, entry) in files.iter().enumerate() {
        let rel = entry.path().strip_prefix(dir).unwrap_or(entry.path());
        let name = rel.to_string_lossy().replace('\\', "/");
        on_event(GatherEvent::Zip {
            name: name.clone(),
            index: i + 1,
            total,
        });
        zip.start_file(&name, options)
            .map_err(|e| Error::Zip(e.to_string()))?;
        let bytes = fs::read(entry.path())?;
        zip.write_all(&bytes)?;
    }
    let mut inner = zip.finish().map_err(|e| Error::Zip(e.to_string()))?;
    inner.flush()?;
    drop(inner);
    Ok(zip_path)
}

fn is_not_empty(err: &std::io::Error) -> bool {
    matches!(err.kind(), ErrorKind::DirectoryNotEmpty)
        || matches!(err.raw_os_error(), Some(66 | 39 | 41 | 145))
}

/// Move off Desktop first so Finder stops planting `.DS_Store`, then delete.
fn remove_output_dir(dir: &Path) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let staging = std::env::temp_dir().join(format!("fonarch-wipe-{}-{stamp}", std::process::id()));
    let target = match fs::rename(dir, &staging) {
        Ok(()) => staging,
        Err(_) => dir.to_path_buf(),
    };
    remove_dir_all_retry(&target)
}

fn remove_dir_all_retry(dir: &Path) -> Result<()> {
    let mut last = None;
    for attempt in 0..12 {
        match fs::remove_dir_all(dir) {
            Ok(()) => return Ok(()),
            Err(_) if !dir.exists() => return Ok(()),
            Err(e) if is_not_empty(&e) => {
                last = Some(e);
                let _ = wipe_once(dir);
                std::thread::sleep(Duration::from_millis(25 * (attempt + 1) as u64));
            }
            Err(e) => return Err(e.into()),
        }
    }
    if !dir.exists() {
        return Ok(());
    }
    Err(last.map(Error::from).unwrap_or_else(|| {
        Error::Io(std::io::Error::new(
            ErrorKind::DirectoryNotEmpty,
            format!("could not remove {}", dir.display()),
        ))
    }))
}

fn wipe_once(dir: &Path) -> std::io::Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in walkdir::WalkDir::new(dir)
        .contents_first(true)
        .follow_links(false)
        .into_iter()
        .flatten()
    {
        let p = entry.path();
        if p == dir {
            continue;
        }
        if entry.file_type().is_dir() {
            let _ = fs::remove_dir(p);
        } else {
            let _ = fs::remove_file(p);
        }
    }
    fs::remove_dir(dir)
}

fn unique_output_dir(parent: &Path) -> PathBuf {
    let now = Local::now();
    let date = now.format("%Y-%m-%d").to_string();
    let time = now.format("%H-%M").to_string();
    let primary = parent.join(format!("FONarch {date}"));
    if !primary.exists() {
        return primary;
    }
    let timed = parent.join(format!("FONarch {date} {time}"));
    if !timed.exists() {
        return timed;
    }
    for i in 2..1000 {
        let p = parent.join(format!("FONarch {date} {time}_{i}"));
        if !p.exists() {
            return p;
        }
    }
    parent.join(format!("FONarch {date} {time}_{}", now.timestamp()))
}

fn unique_file(dir: &Path, file: &str, used: &mut BTreeMap<PathBuf, ()>) -> PathBuf {
    let mut dest = dir.join(file);
    if !dest.exists() && !used.contains_key(&dest) {
        used.insert(dest.clone(), ());
        return dest;
    }
    let stem = dest
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("font")
        .to_string();
    let ext = dest
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("otf")
        .to_string();
    for i in 2..10_000 {
        dest = dir.join(format!("{stem}_{i}.{ext}"));
        if !dest.exists() && !used.contains_key(&dest) {
            used.insert(dest.clone(), ());
            return dest;
        }
    }
    dest
}

fn plan_copies(
    livetype: &Path,
    catalog: &BTreeMap<String, FontMeta>,
    on_event: &mut impl FnMut(GatherEvent),
) -> Vec<Planned> {
    let mut best: BTreeMap<String, (u8, PathBuf, &'static str)> = BTreeMap::new();

    for (bucket, rank) in SOURCE_BUCKETS {
        let dir = livetype.join(bucket);
        let Ok(rd) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in rd.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(id) = id_from_filename(&path) else {
                continue;
            };
            let Some(ext) = magic_ext(&path) else {
                continue;
            };
            match best.get(&id) {
                Some((existing, _, _)) if *existing <= *rank => {}
                _ => {
                    best.insert(id, (*rank, path, ext));
                }
            }
        }
    }

    let mut planned = Vec::with_capacity(best.len());
    for (id, (_rank, src, ext)) in best {
        if let Some(meta) = catalog.get(&id) {
            planned.push(Planned {
                id,
                src,
                family: meta.family_name.clone(),
                variation: meta.variation_name.clone(),
                full_name: meta.full_name.clone(),
                ext,
                fallback: false,
                is_variable: meta.is_variable,
            });
            continue;
        }
        match fallback_names(&src) {
            Some(fb) => planned.push(Planned {
                id,
                src,
                family: fb.family,
                variation: fb.variation,
                full_name: fb.full_name,
                ext,
                fallback: true,
                is_variable: fb.is_variable,
            }),
            None => {
                on_event(GatherEvent::Warn(format!(
                    "no XML row and no name table for id {id}"
                )));
            }
        }
    }

    planned.sort_by(|a, b| {
        a.family
            .cmp(&b.family)
            .then(a.full_name.cmp(&b.full_name))
            .then(a.id.cmp(&b.id))
    });
    planned
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn fixture_livetype(root: &Path) {
        let lt = root.join("livetype");
        fs::create_dir_all(lt.join(".c")).unwrap();
        fs::create_dir_all(lt.join(".r")).unwrap();
        fs::create_dir_all(lt.join(".w")).unwrap();
        fs::create_dir_all(lt.join(".e")).unwrap();
        fs::create_dir_all(lt.join("GudeLivetype")).unwrap();
        fs::write(
            lt.join(".c/entitlements.xml"),
            br#"<?xml version="1.0" encoding="UTF-8"?>
<typekitSyncState>
  <fonts type="array">
    <font>
      <id>169</id>
      <properties>
        <fullName>Proxima Nova Extrabold</fullName>
        <familyName>Proxima Nova</familyName>
        <variationName>Extrabold</variationName>
        <isVariable>false</isVariable>
      </properties>
    </font>
    <font>
      <id>1</id>
      <properties>
        <fullName>Should Stay Encrypted</fullName>
        <familyName>Nope</familyName>
        <variationName>Regular</variationName>
      </properties>
    </font>
  </fonts>
</typekitSyncState>
"#,
        )
        .unwrap();
        fs::write(lt.join(".r/.169.otf"), b"OTTO\0\0\0\0more").unwrap();
        // Encrypted sibling must not be copied.
        fs::write(lt.join(".e/.169"), [0x9c, 0x7b, 0xd7, 0xb5, 1, 2, 3, 4]).unwrap();
        fs::write(lt.join(".e/.1"), [0x9c, 0x7b, 0xd7, 0xb5, 1, 2, 3, 4]).unwrap();
        // Decoy OpenType under GudeLivetype.
        fs::write(lt.join("GudeLivetype/.99.otf"), b"OTTO\0\0\0\0").unwrap();
        // Duplicate id in .w — .r wins.
        fs::write(lt.join(".w/.169.otf"), b"OTTOWWWW").unwrap();
    }

    #[test]
    fn copies_named_ot_skips_encrypted_and_gude() {
        let dir = tempdir().unwrap();
        fixture_livetype(dir.path());
        let out = dir.path().join("desktop");
        fs::create_dir_all(&out).unwrap();
        let report = gather(
            GatherOptions {
                dest_parent: Some(out.clone()),
                dry_run: false,
                livetype: Some(dir.path().join("livetype")),
            },
            |_| {},
        )
        .unwrap();
        assert_eq!(report.copied, 1);
        assert_eq!(report.families, 1);
        assert_eq!(report.fallback, 0);
        let dest = report
            .output
            .join("Proxima_Nova")
            .join("Proxima_Nova_Extrabold.otf");
        assert!(dest.is_file(), "missing {}", dest.display());
        assert_eq!(fs::read(&dest).unwrap(), b"OTTO\0\0\0\0more");
        // dated folder, not a visible working/
        assert!(!report.output.join("working").exists());
        assert!(!report.output.join("DONE").exists());
        assert!(!report.archived);
    }

    #[test]
    fn archive_zips_and_removes_folder() {
        let dir = tempdir().unwrap();
        fixture_livetype(dir.path());
        let out = dir.path().join("desktop");
        fs::create_dir_all(&out).unwrap();
        let report = archive(
            GatherOptions {
                dest_parent: Some(out.clone()),
                dry_run: false,
                livetype: Some(dir.path().join("livetype")),
            },
            |_| {},
        )
        .unwrap();
        assert!(report.archived);
        assert_eq!(
            report.output.extension().and_then(|s| s.to_str()),
            Some("zip")
        );
        assert!(report.output.is_file());
        // leftover folder gone
        let stem = report.output.file_stem().unwrap();
        assert!(!out.join(stem).exists());
        let file = fs::File::open(&report.output).unwrap();
        let mut zip = zip::ZipArchive::new(file).unwrap();
        assert_eq!(zip.len(), 1);
        let inner = zip.by_index(0).unwrap();
        assert!(inner.name().ends_with("Proxima_Nova_Extrabold.otf"));
    }

    #[test]
    fn remove_output_dir_clears_nested_ds_store() {
        let dir = tempdir().unwrap();
        let tree = dir.path().join("FONarch leftover");
        fs::create_dir_all(tree.join("Family")).unwrap();
        fs::write(tree.join(".DS_Store"), b"finder").unwrap();
        fs::write(tree.join("Family/.DS_Store"), b"finder").unwrap();
        fs::write(tree.join("Family/font.otf"), b"OTTO").unwrap();
        remove_output_dir(&tree).unwrap();
        assert!(!tree.exists());
    }

    #[test]
    fn dated_folder_does_not_clobber() {
        let dir = tempdir().unwrap();
        let parent = dir.path();
        let first = unique_output_dir(parent);
        fs::create_dir_all(&first).unwrap();
        let second = unique_output_dir(parent);
        assert_ne!(first, second);
        assert!(second
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("FONarch "));
    }
}
