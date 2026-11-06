//! Software dimming: a full-screen, click-through dark layer above every
//! window. Works on every system (also where brightness can't be changed)
//! and can go darker than a screen's own minimum.

use super::ValleApp;

/// The strongest dimming allowed, so the screen never goes fully black.
pub const MAX_DIM: f32 = 0.85;

impl ValleApp {
    pub(super) fn dim_overlay(&self, ctx: &egui::Context) {
        let level = self.store.dim_level.clamp(0.0, MAX_DIM);
        if self.demo || level <= 0.0 || !self.store.is_installed("brightness") {
            return;
        }
        let builder = egui::ViewportBuilder::default()
            .with_title("Valler dimming")
            .with_decorations(false)
            .with_transparent(true)
            .with_fullscreen(true)
            .with_always_on_top()
            .with_mouse_passthrough(true)
            .with_taskbar(false);
        ctx.show_viewport_immediate(egui::ViewportId::from_hash_of("valler-dim"), builder, |ui, _| {
            let rect = ui.max_rect();
            ui.painter().rect_filled(rect, 0.0, egui::Color32::from_black_alpha((level * 255.0) as u8));
        });
    }
}
