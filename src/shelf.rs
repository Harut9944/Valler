//! Shelf: a place to park files, folders, images, text and links for a
//! moment, then copy, move or open them later. Items are saved, so the shelf
//! survives a restart (pinned items also survive "clear").

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    File,
    Folder,
    Image,
    Text,
    Link,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Item {
    pub id: u64,
    pub kind: Kind,
    #[serde(default)]
    pub path: Option<PathBuf>,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub added: i64,
}

const IMAGE_EXT: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "bmp", "tif", "tiff", "heic", "svg"];

impl Item {
    pub fn title(&self) -> String {
        match &self.path {
            Some(p) => p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| p.display().to_string()),
            None => {
                let first = self.text.lines().find(|l| !l.trim().is_empty()).unwrap_or("").trim();
                if self.kind == Kind::Link {
                    url::Url::parse(first).ok().and_then(|u| u.host_str().map(|h| h.trim_start_matches("www.").to_string())).unwrap_or_else(|| first.to_string())
                } else {
                    first.chars().take(80).collect()
                }
            }
        }
    }

    pub fn exists(&self) -> bool {
        self.path.as_ref().is_none_or(|p| p.exists())
    }

    pub fn size(&self) -> Option<u64> {
        let p = self.path.as_ref()?;
        let m = std::fs::metadata(p).ok()?;
        m.is_file().then_some(m.len())
    }

    pub fn icon(&self) -> &'static str {
        use egui_phosphor::regular as ph;
        match self.kind {
            Kind::File => ph::FILE,
            Kind::Folder => ph::FOLDER,
            Kind::Image => ph::IMAGE,
            Kind::Text => ph::NOTE,
            Kind::Link => ph::LINK,
        }
    }
}

fn next_id(items: &[Item]) -> u64 {
    items.iter().map(|i| i.id).max().unwrap_or(0) + 1
}

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

/// Adds paths, skipping ones already on the shelf. Returns how many were added.
pub fn add_paths(items: &mut Vec<Item>, paths: impl IntoIterator<Item = PathBuf>) -> usize {
    let mut n = 0;
    for p in paths {
        if p.as_os_str().is_empty() || items.iter().any(|i| i.path.as_ref() == Some(&p)) {
            continue;
        }
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        let kind = if p.is_dir() {
            Kind::Folder
        } else if IMAGE_EXT.contains(&ext.as_str()) {
            Kind::Image
        } else {
            Kind::File
        };
        let id = next_id(items);
        items.push(Item { id, kind, path: Some(p), text: String::new(), pinned: false, added: now() });
        n += 1;
    }
    n
}

/// Adds text; a single web address becomes a link.
pub fn add_text(items: &mut Vec<Item>, text: &str) -> bool {
    let t = text.trim();
    if t.is_empty() || items.iter().any(|i| i.path.is_none() && i.text == t) {
        return false;
    }
    let is_link = !t.contains(char::is_whitespace) && url::Url::parse(t).is_ok_and(|u| matches!(u.scheme(), "http" | "https"));
    let id = next_id(items);
    items.push(Item { id, kind: if is_link { Kind::Link } else { Kind::Text }, path: None, text: t.to_string(), pinned: false, added: now() });
    true
}

/// Reads the system clipboard: files copied in a file manager first, then text.
pub fn add_from_clipboard(items: &mut Vec<Item>) -> Result<String, String> {
    let mut cb = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    if let Ok(files) = cb.get().file_list() {
        if !files.is_empty() {
            let n = add_paths(items, files);
            return Ok(if n == 0 { "Those files are already on the Shelf".into() } else { format!("Added {n} item{} to the Shelf", if n == 1 { "" } else { "s" }) });
        }
    }
    match cb.get_text() {
        Ok(t) if !t.trim().is_empty() => {
            if add_text(items, &t) { Ok("Added to the Shelf".into()) } else { Ok("Already on the Shelf".into()) }
        }
        _ => Err("The clipboard is empty".into()),
    }
}

/// Puts the chosen items on the clipboard: files as a file list (paste them in
/// any file manager), text and links as text.
pub fn copy(items: &[&Item]) -> Result<String, String> {
    let mut cb = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    let files: Vec<&PathBuf> = items.iter().filter_map(|i| i.path.as_ref()).filter(|p| p.exists()).collect();
    if !files.is_empty() {
        cb.set().file_list(&files).map_err(|e| e.to_string())?;
        return Ok(format!("Copied {} file{} — paste them in any folder", files.len(), if files.len() == 1 { "" } else { "s" }));
    }
    let text: Vec<&str> = items.iter().filter(|i| i.path.is_none()).map(|i| i.text.as_str()).collect();
    if text.is_empty() {
        return Err("Nothing to copy".into());
    }
    cb.set_text(text.join("\n")).map_err(|e| e.to_string())?;
    Ok("Copied".into())
}

fn unique_target(dir: &Path, name: &std::ffi::OsStr) -> PathBuf {
    let mut t = dir.join(name);
    let stem = Path::new(name).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let ext = Path::new(name).extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
    let mut n = 2;
    while t.exists() {
        t = dir.join(format!("{stem} {n}{ext}"));
        n += 1;
    }
    t
}

fn copy_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if src.is_dir() {
        std::fs::create_dir_all(dst)?;
        for e in std::fs::read_dir(src)? {
            let e = e?;
            copy_recursive(&e.path(), &dst.join(e.file_name()))?;
        }
        Ok(())
    } else {
        std::fs::copy(src, dst).map(|_| ())
    }
}

/// Copies (or moves) the chosen files and folders into `dir`; text items are
/// saved as .txt files and links as .url-style text files.
/// Returns (done, failed, new paths for moved items keyed by item id).
pub fn transfer(items: &[&Item], dir: &Path, moving: bool) -> (usize, usize, Vec<(u64, PathBuf)>) {
    let (mut done, mut failed) = (0, 0);
    let mut moved = Vec::new();
    for it in items {
        match &it.path {
            Some(src) => {
                let Some(name) = src.file_name() else {
                    failed += 1;
                    continue;
                };
                let dst = unique_target(dir, name);
                let r = if moving {
                    std::fs::rename(src, &dst).or_else(|_| copy_recursive(src, &dst).and_then(|_| if src.is_dir() { std::fs::remove_dir_all(src) } else { std::fs::remove_file(src) }))
                } else {
                    copy_recursive(src, &dst)
                };
                match r {
                    Ok(_) => {
                        done += 1;
                        if moving {
                            moved.push((it.id, dst));
                        }
                    }
                    Err(_) => failed += 1,
                }
            }
            None => {
                let base: String = it.title().chars().filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-').take(40).collect();
                let base = if base.trim().is_empty() { "Note".to_string() } else { base.trim().to_string() };
                let dst = unique_target(dir, std::ffi::OsStr::new(&format!("{base}.txt")));
                match std::fs::write(&dst, &it.text) {
                    Ok(_) => done += 1,
                    Err(_) => failed += 1,
                }
            }
        }
    }
    (done, failed, moved)
}

/// Opens a file, folder or link with the system's default app.
pub fn open(item: &Item) {
    let target = match &item.path {
        Some(p) => p.to_string_lossy().to_string(),
        None if item.kind == Kind::Link => item.text.clone(),
        None => return,
    };
    spawn_open(&target);
}

fn spawn_open(target: &str) {
    #[cfg(target_os = "linux")]
    let r = std::process::Command::new("xdg-open").arg(target).spawn();
    #[cfg(target_os = "windows")]
    let r = std::process::Command::new("cmd").args(["/C", "start", ""]).arg(target).spawn();
    #[cfg(target_os = "macos")]
    let r = std::process::Command::new("open").arg(target).spawn();
    if let Ok(mut c) = r {
        std::thread::spawn(move || {
            let _ = c.wait();
        });
    }
}

/// Shows the item in its folder (selected where the file manager supports it).
pub fn reveal(item: &Item) {
    let Some(p) = &item.path else { return };
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer").arg(format!("/select,{}", p.display())).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg("-R").arg(p).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        // The freedesktop FileManager1 interface selects the file in Files, Dolphin and others.
        let uri = url::Url::from_file_path(p).map(|u| u.to_string()).unwrap_or_default();
        let ok = std::process::Command::new("dbus-send")
            .args([
                "--session",
                "--dest=org.freedesktop.FileManager1",
                "--type=method_call",
                "/org/freedesktop/FileManager1",
                "org.freedesktop.FileManager1.ShowItems",
                &format!("array:string:{uri}"),
                "string:",
            ])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !ok {
            if let Some(parent) = p.parent() {
                spawn_open(&parent.to_string_lossy());
            }
        }
    }
}

/// A short summary like "3 files, 1 link".
pub fn summary(items: &[Item]) -> String {
    let count = |k: &[Kind]| items.iter().filter(|i| k.contains(&i.kind)).count();
    let mut parts = Vec::new();
    for (n, one, many) in [
        (count(&[Kind::File, Kind::Folder]), "file", "files"),
        (count(&[Kind::Image]), "image", "images"),
        (count(&[Kind::Text]), "note", "notes"),
        (count(&[Kind::Link]), "link", "links"),
    ] {
        if n > 0 {
            parts.push(format!("{n} {}", if n == 1 { one } else { many }));
        }
    }
    if parts.is_empty() { "Empty".into() } else { parts.join(", ") }
}

pub fn demo() -> Vec<Item> {
    let mk = |id, kind, path: Option<&str>, text: &str, pinned| Item { id, kind, path: path.map(PathBuf::from), text: text.into(), pinned, added: 0 };
    vec![
        mk(1, Kind::File, Some("Quarterly report.pdf"), "", false),
        mk(2, Kind::Image, Some("Screenshot.png"), "", false),
        mk(3, Kind::Folder, Some("Project assets"), "", true),
        mk(4, Kind::Link, None, "https://www.example.com/article", false),
        mk(5, Kind::Text, None, "Call back after 3 pm", false),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn adds_and_detects_kinds() {
        let mut v = Vec::new();
        assert!(add_text(&mut v, "https://example.com/a"));
        assert!(add_text(&mut v, "hello there"));
        assert!(!add_text(&mut v, "hello there"));
        assert_eq!(v[0].kind, Kind::Link);
        assert_eq!(v[1].kind, Kind::Text);
        assert_eq!(add_paths(&mut v, [PathBuf::from("/tmp/x.png")]), 1);
        assert_eq!(v[2].kind, Kind::Image);
        assert_eq!(v[0].title(), "example.com");
        assert_eq!(summary(&v), "1 image, 1 note, 1 link");
    }
}

#[cfg(test)]
mod feature_tests {
    use super::*;
    use crate::test_util::TempDir;

    fn setup() -> (TempDir, Vec<Item>) {
        let t = TempDir::new("shelf");
        std::fs::write(t.join("a.txt"), "hello").unwrap();
        std::fs::create_dir_all(t.join("folder/inner")).unwrap();
        std::fs::write(t.join("folder/inner/b.txt"), "deep").unwrap();
        let mut items = Vec::new();
        add_paths(&mut items, [t.join("a.txt"), t.join("folder")]);
        (t, items)
    }

    #[test]
    fn detects_files_and_folders_and_skips_duplicates() {
        let (t, mut items) = setup();
        assert_eq!(items[0].kind, Kind::File);
        assert_eq!(items[1].kind, Kind::Folder);
        assert_eq!(add_paths(&mut items, [t.join("a.txt")]), 0);
        assert_eq!(items[0].title(), "a.txt");
        assert_eq!(items[0].size(), Some(5));
        assert!(items[0].exists());
        assert_ne!(items[0].id, items[1].id);
    }

    #[test]
    fn copies_files_and_folders_into_a_folder_without_overwriting() {
        let (t, items) = setup();
        let dst = TempDir::new("shelf-dst");
        let refs: Vec<&Item> = items.iter().collect();
        let (done, failed, moved) = transfer(&refs, &dst.0, false);
        assert_eq!((done, failed), (2, 0));
        assert!(moved.is_empty());
        assert_eq!(std::fs::read_to_string(dst.join("a.txt")).unwrap(), "hello");
        assert_eq!(std::fs::read_to_string(dst.join("folder/inner/b.txt")).unwrap(), "deep");
        assert!(t.join("a.txt").exists(), "copying keeps the original");
        // A second copy gets a new name instead of replacing the first.
        transfer(&refs[..1], &dst.0, false);
        assert!(dst.join("a 2.txt").exists());
    }

    #[test]
    fn moves_files_and_reports_new_locations() {
        let (t, items) = setup();
        let dst = TempDir::new("shelf-move");
        let refs: Vec<&Item> = items.iter().collect();
        let (done, failed, moved) = transfer(&refs, &dst.0, true);
        assert_eq!((done, failed), (2, 0));
        assert!(!t.join("a.txt").exists());
        assert!(!t.join("folder").exists());
        assert_eq!(moved.len(), 2);
        assert!(moved.iter().all(|(_, p)| p.exists()));
    }

    #[test]
    fn text_items_are_saved_as_text_files() {
        let dst = TempDir::new("shelf-text");
        let mut items = Vec::new();
        add_text(&mut items, "Buy milk\nand bread");
        let refs: Vec<&Item> = items.iter().collect();
        let (done, _, _) = transfer(&refs, &dst.0, false);
        assert_eq!(done, 1);
        assert_eq!(std::fs::read_to_string(dst.join("Buy milk.txt")).unwrap(), "Buy milk\nand bread");
    }

    #[test]
    fn missing_files_are_detected() {
        let (t, items) = setup();
        std::fs::remove_file(t.join("a.txt")).unwrap();
        assert!(!items[0].exists());
        assert_eq!(items[0].size(), None);
    }

    #[test]
    fn summary_and_demo() {
        assert_eq!(summary(&[]), "Empty");
        let d = demo();
        assert_eq!(summary(&d), "2 files, 1 image, 1 note, 1 link");
        assert!(d.iter().any(|i| i.pinned));
    }

    #[test]
    fn items_survive_saving() {
        let (_t, items) = setup();
        let json = serde_json::to_string(&items).unwrap();
        let back: Vec<Item> = serde_json::from_str(&json).unwrap();
        assert_eq!(back.len(), 2);
        assert_eq!(back[1].kind, Kind::Folder);
    }
}
