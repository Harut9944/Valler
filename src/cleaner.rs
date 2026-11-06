//! Cleaner: measures caches, logs and temporary files in the places each
//! system keeps them, and moves the chosen ones to the trash (recoverable).

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct Item {
    pub category: &'static str,
    pub path: PathBuf,
    pub size: u64,
    pub selected: bool,
}

#[derive(Clone, Default)]
pub enum ScanState {
    #[default]
    Idle,
    Scanning,
    Done(Vec<Item>),
    Cleaned { freed: u64, failed: usize },
}

pub type Shared = Arc<Mutex<ScanState>>;

fn roots() -> Vec<(&'static str, PathBuf)> {
    let mut v = Vec::new();
    #[cfg(target_os = "linux")]
    {
        if let Some(c) = dirs::cache_dir() {
            v.push(("App caches", c));
        }
        if let Some(h) = dirs::home_dir() {
            v.push(("Thumbnails", h.join(".cache/thumbnails")));
            v.push(("Trash leftovers", h.join(".local/share/Trash/files")));
        }
    }
    #[cfg(target_os = "windows")]
    {
        v.push(("Temporary files", std::env::temp_dir()));
        if let Some(l) = dirs::data_local_dir() {
            v.push(("Crash dumps", l.join("CrashDumps")));
            v.push(("Explorer thumbnails", l.join("Microsoft\\Windows\\Explorer")));
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(c) = dirs::cache_dir() {
            v.push(("App caches", c));
        }
        if let Some(h) = dirs::home_dir() {
            v.push(("Logs", h.join("Library/Logs")));
        }
    }
    v
}

fn dir_size(p: &Path, depth: u32) -> u64 {
    let Ok(meta) = std::fs::symlink_metadata(p) else { return 0 };
    if meta.is_file() {
        return meta.len();
    }
    if !meta.is_dir() || depth > 24 {
        return 0;
    }
    std::fs::read_dir(p)
        .map(|rd| rd.flatten().map(|e| dir_size(&e.path(), depth + 1)).sum())
        .unwrap_or(0)
}

pub fn scan(state: Shared, ctx: egui::Context) {
    if let Ok(mut s) = state.lock() {
        *s = ScanState::Scanning;
    }
    std::thread::spawn(move || {
        let mut items = Vec::new();
        for (cat, root) in roots() {
            let Ok(rd) = std::fs::read_dir(&root) else { continue };
            for e in rd.flatten() {
                let path = e.path();
                // Never touch Valler's own data.
                if path.file_name().is_some_and(|n| n.to_string_lossy().to_lowercase().contains("valler")) {
                    continue;
                }
                let size = dir_size(&path, 0);
                if size >= 512 * 1024 {
                    items.push(Item { category: cat, path, size, selected: false });
                }
            }
        }
        items.sort_by(|a, b| b.size.cmp(&a.size));
        if let Ok(mut s) = state.lock() {
            *s = ScanState::Done(items);
        }
        ctx.request_repaint();
    });
}

pub fn clean(state: Shared, items: Vec<Item>, ctx: egui::Context) {
    if let Ok(mut s) = state.lock() {
        *s = ScanState::Scanning;
    }
    std::thread::spawn(move || {
        let mut freed = 0;
        let mut failed = 0;
        for it in items.iter().filter(|i| i.selected) {
            match trash::delete(&it.path) {
                Ok(_) => freed += it.size,
                Err(_) => failed += 1,
            }
        }
        if let Ok(mut s) = state.lock() {
            *s = ScanState::Cleaned { freed, failed };
        }
        ctx.request_repaint();
    });
}

pub fn demo() -> Vec<Item> {
    vec![
        Item { category: "App caches", path: PathBuf::from("~/.cache/browser"), size: 734_003_200, selected: true },
        Item { category: "Thumbnails", path: PathBuf::from("~/.cache/thumbnails"), size: 88_080_384, selected: false },
    ]
}
