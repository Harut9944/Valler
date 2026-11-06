//! Work the app does between frames: polling the sound server for the mixer
//! and collecting new clipboard entries.

use crate::services::clipboard::WatchConfig;
use crate::services::mixer::{self};
use crate::core::store::Store;
use crate::services::clipboard::history;
use std::time::{Duration, Instant};
use super::*;

impl ValleApp {
    pub(super) fn poll_mixer(&mut self, ctx: &egui::Context) {
        if self.demo {
            if let Ok(mut m) = self.mixer.lock()
                && !m.available && self.mixer_last.is_none() {
                    let real = mixer::read();
                    *m = if real.available && !real.streams.is_empty() { real } else { mixer::demo() };
                }
            self.mixer_last = Some(Instant::now());
            return;
        }
        let due = self.mixer_last.is_none_or(|t| t.elapsed() > Duration::from_millis(1500));
        let holding = self.mixer_hold.is_some_and(|t| t.elapsed() < Duration::from_millis(1200));
        let busy = self.mixer_busy.lock().map(|b| *b).unwrap_or(true);
        if due && !holding && !busy {
            self.mixer_last = Some(Instant::now());
            let out = self.mixer.clone();
            let flag = self.mixer_busy.clone();
            let ctx = ctx.clone();
            if let Ok(mut b) = flag.lock() {
                *b = true;
            }
            std::thread::spawn(move || {
                let st = mixer::read();
                if let Ok(mut m) = out.lock() {
                    *m = st;
                }
                if let Ok(mut b) = flag.lock() {
                    *b = false;
                }
                ctx.request_repaint();
            });
        }
        ctx.request_repaint_after(Duration::from_millis(1500));
    }

    pub(super) fn drain_clipboard(&mut self) {
        let mut changed = false;
        let mut unused = Vec::new();
        if let Some(w) = &self.clip {
            while let Ok(clip) = w.rx.try_recv() {
                if !self.store.is_on("clipboard") {
                    continue;
                }
                let entry = clip.into_entry(chrono::Utc::now().timestamp());
                unused.extend(history::add(&mut self.store.clip_history, entry, self.store.clip_max_items.max(10)));
                changed = true;
            }
        }
        forget_images(unused);
        if changed {
            self.mark_dirty();
        }
    }
}

/// Deletes saved clipboard images no entry uses any more.
pub(crate) fn forget_images(paths: Vec<std::path::PathBuf>) {
    let dir = crate::services::clipboard::images::dir();
    for p in paths {
        // Only ever touch Valler's own clipboard folder.
        if p.starts_with(&dir) {
            let _ = std::fs::remove_file(p);
        }
    }
}

pub(super) fn watch_config(s: &Store) -> WatchConfig {
    WatchConfig {
        watch: s.is_on("clipboard"),
        images: s.clip_images,
        auto_clean_urls: s.is_on("url_clean"),
        extra_params: s.url_extra_params.clone(),
        auto_clear_secs: if s.is_on("clip_autoclear") { s.clip_auto_clear_secs.max(5) } else { 0 },
    }
}
