//! Clipboard history rules: what an entry is (text, image or files), how a
//! new copy joins the list (moved to the top if already there, pin kept),
//! how many unpinned entries are kept, and which saved images become unused.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ClipKind {
    #[default]
    Text,
    Image,
    Files,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClipEntry {
    /// The text, or a description for images ("Image 640 × 480") and files.
    pub text: String,
    pub pinned: bool,
    /// Unix seconds when copied.
    pub at: i64,
    #[serde(default)]
    pub kind: ClipKind,
    /// Saved PNG for image entries.
    #[serde(default)]
    pub image: Option<PathBuf>,
    #[serde(default)]
    pub files: Vec<PathBuf>,
}

impl ClipEntry {
    pub fn text(text: impl Into<String>, at: i64) -> Self {
        Self { text: text.into(), pinned: false, at, kind: ClipKind::Text, image: None, files: Vec::new() }
    }

    pub fn image(path: PathBuf, width: usize, height: usize, at: i64) -> Self {
        Self { text: format!("Image {width} × {height}"), pinned: false, at, kind: ClipKind::Image, image: Some(path), files: Vec::new() }
    }

    pub fn files(files: Vec<PathBuf>, at: i64) -> Self {
        let names: Vec<String> = files.iter().map(|f| f.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| f.display().to_string())).collect();
        let text = match names.len() {
            0 => "No files".to_string(),
            1 => names[0].clone(),
            n => format!("{} and {} more", names[0], n - 1),
        };
        Self { text, pinned: false, at, kind: ClipKind::Files, image: None, files }
    }

    /// Two copies of the same thing (used to move repeats to the top).
    pub fn same_content(&self, other: &ClipEntry) -> bool {
        self.kind == other.kind
            && match self.kind {
                ClipKind::Text => self.text == other.text,
                ClipKind::Image => self.image == other.image,
                ClipKind::Files => self.files == other.files,
            }
    }

    /// Words the search box matches against.
    pub fn haystack(&self) -> String {
        let mut s = self.text.to_lowercase();
        match self.kind {
            ClipKind::Image => s.push_str(" image picture screenshot"),
            ClipKind::Files => {
                s.push_str(" file files");
                for f in &self.files {
                    s.push(' ');
                    s.push_str(&f.to_string_lossy().to_lowercase());
                }
            }
            ClipKind::Text => {}
        }
        s
    }
}

/// Adds a new copy at the top. A repeat moves up and keeps its pin. Only the
/// newest `max_unpinned` unpinned entries stay; pinned ones always stay.
/// Returns image files no longer used by any entry (safe to delete).
pub fn add(history: &mut Vec<ClipEntry>, mut entry: ClipEntry, max_unpinned: usize) -> Vec<PathBuf> {
    if let Some(i) = history.iter().position(|e| e.same_content(&entry)) {
        entry.pinned = history.remove(i).pinned;
    }
    history.insert(0, entry);
    let mut unpinned = 0;
    let mut dropped = Vec::new();
    history.retain(|e| {
        if e.pinned {
            return true;
        }
        unpinned += 1;
        let keep = unpinned <= max_unpinned.max(1);
        if !keep {
            dropped.push(e.clone());
        }
        keep
    });
    unused_images(history, &dropped)
}

/// Removes unpinned entries; returns their now-unused image files.
pub fn clear_unpinned(history: &mut Vec<ClipEntry>) -> Vec<PathBuf> {
    let dropped: Vec<ClipEntry> = history.iter().filter(|e| !e.pinned).cloned().collect();
    history.retain(|e| e.pinned);
    unused_images(history, &dropped)
}

/// Removes one entry; returns its image file if nothing else uses it.
pub fn remove(history: &mut Vec<ClipEntry>, index: usize) -> Vec<PathBuf> {
    if index >= history.len() {
        return vec![];
    }
    let e = history.remove(index);
    unused_images(history, &[e])
}

fn unused_images(history: &[ClipEntry], dropped: &[ClipEntry]) -> Vec<PathBuf> {
    dropped
        .iter()
        .filter_map(|e| e.image.clone())
        .filter(|p| !history.iter().any(|h| h.image.as_ref() == Some(p)))
        .collect()
}

/// Indices matching a search, pinned entries first, newest first within each.
pub fn search(history: &[ClipEntry], query: &str) -> Vec<usize> {
    let q = query.trim().to_lowercase();
    let mut v: Vec<usize> = (0..history.len()).filter(|i| q.is_empty() || history[*i].haystack().contains(&q)).collect();
    v.sort_by_key(|i| !history[*i].pinned);
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(h: &[ClipEntry]) -> Vec<&str> {
        h.iter().map(|e| e.text.as_str()).collect()
    }

    #[test]
    fn repeats_move_to_the_top_and_keep_their_pin() {
        let mut h = Vec::new();
        add(&mut h, ClipEntry::text("a", 1), 10);
        add(&mut h, ClipEntry::text("b", 2), 10);
        h[1].pinned = true; // "a"
        add(&mut h, ClipEntry::text("a", 3), 10);
        assert_eq!(texts(&h), ["a", "b"]);
        assert!(h[0].pinned);
        assert_eq!(h[0].at, 3);
    }

    #[test]
    fn keeps_only_the_newest_unpinned_entries() {
        let mut h = Vec::new();
        add(&mut h, ClipEntry::text("keep me", 0), 2);
        h[0].pinned = true;
        for (i, t) in ["1", "2", "3"].iter().enumerate() {
            add(&mut h, ClipEntry::text(*t, i as i64), 2);
        }
        assert_eq!(texts(&h), ["3", "2", "keep me"]);
    }

    #[test]
    fn dropped_images_are_reported_for_deletion() {
        let mut h = Vec::new();
        let gone = add(&mut h, ClipEntry::image("a.png".into(), 10, 10, 0), 1);
        assert!(gone.is_empty());
        let gone = add(&mut h, ClipEntry::text("x", 1), 1);
        assert_eq!(gone, vec![PathBuf::from("a.png")]);
        // The same image copied twice is one entry, and stays while it's used.
        let mut h = Vec::new();
        add(&mut h, ClipEntry::image("b.png".into(), 1, 1, 0), 5);
        add(&mut h, ClipEntry::image("b.png".into(), 1, 1, 1), 5);
        assert_eq!(h.len(), 1);
        assert_eq!(remove(&mut h, 0), vec![PathBuf::from("b.png")]);
        assert!(remove(&mut h, 3).is_empty());
    }

    #[test]
    fn clearing_keeps_pinned_entries() {
        let mut h = vec![ClipEntry::image("p.png".into(), 1, 1, 0), ClipEntry::image("q.png".into(), 1, 1, 0)];
        h[0].pinned = true;
        assert_eq!(clear_unpinned(&mut h), vec![PathBuf::from("q.png")]);
        assert_eq!(h.len(), 1);
    }

    #[test]
    fn files_entries_describe_and_search_their_files() {
        let e = ClipEntry::files(vec!["/home/me/report.pdf".into(), "/home/me/photo.jpg".into()], 0);
        assert_eq!(e.text, "report.pdf and 1 more");
        assert_eq!(e.kind, ClipKind::Files);
        let h = vec![ClipEntry::text("hello", 0), e, ClipEntry::image("x.png".into(), 4, 3, 0)];
        assert_eq!(search(&h, "photo"), vec![1]);
        assert_eq!(search(&h, "screenshot"), vec![2]);
        assert_eq!(search(&h, ""), vec![0, 1, 2]);
        assert_eq!(h[2].text, "Image 4 × 3");
    }

    #[test]
    fn older_saved_history_loads_as_text() {
        let e: ClipEntry = serde_json::from_str(r#"{"text":"hi","pinned":true,"at":5}"#).unwrap();
        assert_eq!(e.kind, ClipKind::Text);
        assert!(e.image.is_none() && e.files.is_empty());
    }
}
