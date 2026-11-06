//! Image thumbnails for any screen (Shelf tiles, clipboard images): decoded
//! on a background thread, uploaded to the GPU on the next frame and cached
//! by file path.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

const MAX_SIDE: u32 = 192;

#[derive(Default)]
pub struct Thumbs {
    ready: HashMap<PathBuf, egui::TextureHandle>,
    requested: HashSet<PathBuf>,
    decoded: Arc<Mutex<Vec<(PathBuf, egui::ColorImage)>>>,
}

impl Thumbs {
    /// The thumbnail if it's ready; otherwise starts decoding it (once).
    pub fn get(&mut self, ctx: &egui::Context, path: &Path) -> Option<egui::TextureHandle> {
        let finished: Vec<(PathBuf, egui::ColorImage)> = self.decoded.lock().map(|mut q| std::mem::take(&mut *q)).unwrap_or_default();
        for (p, img) in finished {
            let tex = ctx.load_texture(format!("thumb-{}", p.display()), img, egui::TextureOptions::LINEAR);
            self.ready.insert(p, tex);
        }
        if let Some(t) = self.ready.get(path) {
            return Some(t.clone());
        }
        if self.requested.insert(path.to_path_buf()) {
            let (path, out, ctx) = (path.to_path_buf(), self.decoded.clone(), ctx.clone());
            std::thread::spawn(move || {
                if let Ok(img) = image::open(&path) {
                    let t = img.thumbnail(MAX_SIDE, MAX_SIDE).to_rgba8();
                    let ci = egui::ColorImage::from_rgba_unmultiplied([t.width() as usize, t.height() as usize], t.as_raw());
                    if let Ok(mut q) = out.lock() {
                        q.push((path, ci));
                    }
                    ctx.request_repaint();
                }
            });
        }
        None
    }

    /// Draws a thumbnail fitted into `rect`; returns false if it isn't ready yet.
    pub fn paint(&mut self, ui: &egui::Ui, path: &Path, rect: egui::Rect) -> bool {
        let Some(t) = self.get(ui.ctx(), path) else { return false };
        let size = t.size_vec2();
        let scale = (rect.width() / size.x).min(rect.height() / size.y);
        let r = egui::Rect::from_center_size(rect.center(), size * scale);
        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        ui.painter().image(t.id(), r, uv, egui::Color32::WHITE);
        true
    }
}
