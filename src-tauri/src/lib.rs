use std::path::PathBuf;
use std::sync::Mutex;

use fonarch_core::{archive, gather, GatherEvent, GatherOptions, GatherReport};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

struct Busy(Mutex<bool>);

struct BusyGuard<'a> {
    lock: &'a Mutex<bool>,
}

impl Drop for BusyGuard<'_> {
    fn drop(&mut self) {
        if let Ok(mut g) = self.lock.lock() {
            *g = false;
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum Payload {
    Status {
        text: String,
    },
    Found {
        path: String,
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
    Zip {
        name: String,
        index: usize,
        total: usize,
    },
    Warn {
        text: String,
    },
    Done {
        output: String,
        copied: usize,
        families: usize,
        archived: bool,
    },
    Failed {
        message: String,
    },
}

fn emit(app: &AppHandle, payload: Payload) {
    let _ = app.emit("fonarch", payload);
}

fn from_event(ev: GatherEvent) -> Payload {
    match ev {
        GatherEvent::Status(text) => Payload::Status { text },
        GatherEvent::Found { path, via } => Payload::Found {
            path: path.display().to_string(),
            via,
        },
        GatherEvent::Plan { total, families } => Payload::Plan { total, families },
        GatherEvent::Font { name, index, total } => Payload::Font { name, index, total },
        GatherEvent::Zip { name, index, total } => Payload::Zip { name, index, total },
        GatherEvent::Warn(text) => Payload::Warn { text },
        GatherEvent::Done { report } => from_report(report),
    }
}

fn from_report(report: GatherReport) -> Payload {
    Payload::Done {
        output: report.output.display().to_string(),
        copied: report.copied,
        families: report.families,
        archived: report.archived,
    }
}

#[tauri::command]
fn desktop_path() -> Result<String, String> {
    dirs::desktop_dir()
        .map(|p| p.display().to_string())
        .ok_or_else(|| "could not find the Desktop folder".into())
}

fn origin_ok(url: &str, origin: &str) -> bool {
    url == origin || url.starts_with(&format!("{origin}/"))
}

fn url_allowed(url: &str) -> bool {
    let url = url.trim();
    let apache = "https://www.apache.org/licenses/LICENSE-2.0";
    url == apache
        || url.starts_with(apache)
        || origin_ok(url, "https://www.kreativekorp.com")
        || origin_ok(url, "https://mdeanjones.net")
        || origin_ok(url, "https://www.mdeanjones.net")
}

#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    if !url_allowed(&url) {
        return Err("url not allowed".into());
    }
    open::that(&url).map_err(|e| e.to_string())
}

#[tauri::command]
fn run_job(
    app: AppHandle,
    busy: State<'_, Busy>,
    mode: String,
    dest: Option<String>,
) -> Result<(), String> {
    {
        let mut g = busy.0.lock().map_err(|e| e.to_string())?;
        if *g {
            return Err("already running".into());
        }
        *g = true;
    }

    let dest_parent = match dest {
        Some(s) if !s.trim().is_empty() => {
            let p = PathBuf::from(s);
            if !p.is_dir() {
                *busy.0.lock().map_err(|e| e.to_string())? = false;
                return Err(format!("not a folder: {}", p.display()));
            }
            Some(p)
        }
        _ => None,
    };

    let app2 = app.clone();
    std::thread::spawn(move || {
        let handle = app2.state::<Busy>();
        let _guard = BusyGuard { lock: &handle.0 };
        let opts = GatherOptions {
            dest_parent,
            ..GatherOptions::default()
        };
        let emit_ev = |ev: GatherEvent| emit(&app2, from_event(ev));
        let result = match mode.as_str() {
            "archive" => archive(opts, emit_ev),
            _ => gather(opts, emit_ev),
        };
        if let Err(e) = result {
            emit(
                &app2,
                Payload::Failed {
                    message: e.to_string(),
                },
            );
        }
    });
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Busy(Mutex::new(false)))
        .invoke_handler(tauri::generate_handler![run_job, desktop_path, open_url])
        .run(tauri::generate_context!())
        .expect("FONarch failed to start");
}

#[cfg(test)]
mod tests {
    use super::url_allowed;

    #[test]
    fn url_allowlist() {
        assert!(url_allowed("https://www.apache.org/licenses/LICENSE-2.0"));
        assert!(url_allowed("https://www.apache.org/licenses/LICENSE-2.0.html"));
        assert!(url_allowed("https://www.kreativekorp.com/"));
        assert!(url_allowed("https://www.kreativekorp.com"));
        assert!(url_allowed("https://mdeanjones.net/"));
        assert!(url_allowed("https://mdeanjones.net"));
        assert!(url_allowed("https://www.mdeanjones.net/"));
        assert!(!url_allowed("http://www.kreativekorp.com/"));
        assert!(!url_allowed("https://example.com/"));
        assert!(!url_allowed("https://www.kreativekorp.com.evil.example/"));
        assert!(!url_allowed("https://mdeanjones.net.evil.example/"));
    }
}
