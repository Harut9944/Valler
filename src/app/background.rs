//! Work the app does between frames: polling the sound server for the mixer
//! and collecting new clipboard entries.

use crate::services::clipboard::WatchConfig;
use crate::services::mixer::{self};
use crate::core::store::{ClipEntry, Store};
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
        if let Some(w) = &self.clip {
            while let Ok(text) = w.rx.try_recv() {
                if !self.store.is_on("clipboard") {
                    continue;
                }
                let h = &mut self.store.clip_history;
                let pinned = h.iter().position(|e| e.text == text).map(|i| h.remove(i).pinned).unwrap_or(false);
                h.insert(0, ClipEntry { text, pinned, at: chrono::Utc::now().timestamp() });
                let max = self.store.clip_max_items.max(10);
                let mut unpinned = 0;
                h.retain(|e| {
                    if e.pinned {
                        return true;
                    }
                    unpinned += 1;
                    unpinned <= max
                });
                changed = true;
            }
        }
        if changed {
            self.mark_dirty();
        }
    }
}

pub(super) fn watch_config(s: &Store) -> WatchConfig {
    WatchConfig {
        watch: s.is_on("clipboard"),
        auto_clean_urls: s.is_on("url_clean"),
        extra_params: s.url_extra_params.clone(),
        auto_clear_secs: if s.is_on("clip_autoclear") { s.clip_auto_clear_secs.max(5) } else { 0 },
    }
}
