//! Installed applications for the command bar: .desktop entries on Linux,
//! Start Menu shortcuts on Windows and .app bundles on macOS.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct AppEntry {
    pub name: String,
    pub detail: String,
    #[cfg_attr(target_os = "linux", allow(dead_code))]
    pub launch: PathBuf,
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    pub exec: String,
}

pub type Shared = Arc<Mutex<Option<Vec<AppEntry>>>>;

/// Scans once on a background thread; the list fills in when ready.
pub fn scan(out: Shared, ctx: egui::Context) {
    std::thread::spawn(move || {
        let mut list = list_apps();
        list.sort_by_key(|a| a.name.to_lowercase());
        list.dedup_by(|a, b| a.name.eq_ignore_ascii_case(&b.name));
        if let Ok(mut o) = out.lock() {
            *o = Some(list);
        }
        ctx.request_repaint();
    });
}

#[cfg(target_os = "linux")]
fn list_apps() -> Vec<AppEntry> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Some(d) = dirs::data_dir() {
        dirs.push(d.join("applications"));
    }
    let sys = std::env::var("XDG_DATA_DIRS").unwrap_or_else(|_| "/usr/local/share:/usr/share".into());
    for d in sys.split(':').filter(|d| !d.is_empty()) {
        dirs.push(Path::new(d).join("applications"));
    }
    dirs.push(PathBuf::from("/var/lib/flatpak/exports/share/applications"));
    if let Some(h) = dirs::home_dir() {
        dirs.push(h.join(".local/share/flatpak/exports/share/applications"));
    }
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default().to_uppercase();
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for d in dirs {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) != Some("desktop") {
                continue;
            }
            let id = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            if !seen.insert(id) {
                continue; // user entries override system ones
            }
            if let Some(a) = parse_desktop(&p, &desktop) {
                out.push(a);
            }
        }
    }
    out
}

#[cfg(target_os = "linux")]
fn parse_desktop(path: &Path, desktop: &str) -> Option<AppEntry> {
    let text = std::fs::read_to_string(path).ok()?;
    let mut in_entry = false;
    let (mut name, mut exec, mut comment) = (None, None, String::new());
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_entry = line == "[Desktop Entry]";
            continue;
        }
        if !in_entry {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else { continue };
        match k.trim() {
            "Name" => name = Some(v.trim().to_string()),
            "Exec" => exec = Some(v.trim().to_string()),
            "Comment" if comment.is_empty() => comment = v.trim().to_string(),
            "GenericName" if comment.is_empty() => comment = v.trim().to_string(),
            "Type" if v.trim() != "Application" => return None,
            "NoDisplay" | "Hidden" if v.trim() == "true" => return None,
            "OnlyShowIn" if !desktop.is_empty() && !v.split(';').any(|d| !d.is_empty() && desktop.contains(&d.to_uppercase())) => return None,
            "NotShowIn" if !desktop.is_empty() && v.split(';').any(|d| !d.is_empty() && desktop.contains(&d.to_uppercase())) => return None,
            _ => {}
        }
    }
    Some(AppEntry { name: name?, detail: comment, launch: path.to_path_buf(), exec: exec? })
}

#[cfg(target_os = "windows")]
fn list_apps() -> Vec<AppEntry> {
    let mut roots = Vec::new();
    if let Ok(p) = std::env::var("ProgramData") {
        roots.push(PathBuf::from(p).join(r"Microsoft\Windows\Start Menu\Programs"));
    }
    if let Ok(p) = std::env::var("APPDATA") {
        roots.push(PathBuf::from(p).join(r"Microsoft\Windows\Start Menu\Programs"));
    }
    let mut out = Vec::new();
    for r in roots {
        walk(&r, 0, &mut |p| {
            if p.extension().and_then(|x| x.to_str()).is_some_and(|x| x.eq_ignore_ascii_case("lnk")) {
                let name = p.file_stem().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                let lname = name.to_lowercase();
                if lname.contains("uninstall") || lname.contains("readme") {
                    return;
                }
                let folder = p.parent().and_then(|f| f.file_name()).map(|f| f.to_string_lossy().to_string()).unwrap_or_default();
                out.push(AppEntry { name, detail: folder, launch: p.to_path_buf(), exec: String::new() });
            }
        });
    }
    out
}

#[cfg(target_os = "macos")]
fn list_apps() -> Vec<AppEntry> {
    let mut roots = vec![
        PathBuf::from("/Applications"),
        PathBuf::from("/Applications/Utilities"),
        PathBuf::from("/System/Applications"),
        PathBuf::from("/System/Applications/Utilities"),
    ];
    if let Some(h) = dirs::home_dir() {
        roots.push(h.join("Applications"));
    }
    let mut out = Vec::new();
    for r in roots {
        let Ok(rd) = std::fs::read_dir(&r) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) == Some("app") {
                let name = p.file_stem().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                out.push(AppEntry { name, detail: r.display().to_string(), launch: p, exec: String::new() });
            }
        }
    }
    out
}

#[cfg(target_os = "windows")]
fn walk(dir: &Path, depth: u32, f: &mut dyn FnMut(&Path)) {
    if depth > 4 {
        return;
    }
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, depth + 1, f);
        } else {
            f(&p);
        }
    }
}

/// Starts the app without waiting for it.
pub fn launch(app: &AppEntry) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    let mut cmd = {
        // Drop field codes like %U and %f that only make sense with files.
        let exec: String = app
            .exec
            .split_whitespace()
            .filter(|t| !(t.len() == 2 && t.starts_with('%')))
            .collect::<Vec<_>>()
            .join(" ");
        let mut c = std::process::Command::new("sh");
        c.args(["-c", &format!("{exec} &")]);
        c
    };
    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = std::process::Command::new("cmd");
        c.args(["/C", "start", ""]).arg(&app.launch);
        c
    };
    #[cfg(target_os = "macos")]
    let mut cmd = {
        let mut c = std::process::Command::new("open");
        c.arg(&app.launch);
        c
    };
    let mut child = cmd.spawn().map_err(|e| e.to_string())?;
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}
