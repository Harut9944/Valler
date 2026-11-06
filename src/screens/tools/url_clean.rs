//! Clean URL.

use crate::app::ValleApp;
use crate::ui::{self, caption, card, title};
use crate::services::urlclean;
use egui::{RichText, Ui};
use egui_phosphor::regular as ph;

// ------------------------------------------------------------- clean url

pub(crate) fn url_clean(app: &mut ValleApp, ui: &mut Ui, ctx: &egui::Context) {
    card(ui, |ui| {
        ui.label(title("Clean a link"));
        let r = ui.add(egui::TextEdit::singleline(&mut app.url_input).hint_text("Paste a link").desired_width(f32::INFINITY).margin(egui::vec2(10.0, 7.0)));
        let enter = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        ui.horizontal(|ui| {
            if ui::primary_button(ui, "Clean").clicked() || enter {
                run_clean(app);
            }
            if ui::soft_button(ui, "Clean the link on my clipboard").clicked() {
                let latest = arboard::Clipboard::new().ok().and_then(|mut c| c.get_text().ok()).or_else(|| app.store.clip_history.first().map(|e| e.text.clone()));
                app.url_input = latest.unwrap_or_default();
                run_clean(app);
                if let Some((u, removed)) = app.url_result.clone()
                    && !removed.is_empty() {
                        app.copy_text(ctx, u);
                    }
            }
        });
        if let Some(e) = &app.url_error {
            ui.label(RichText::new(e).color(ui::ORANGE).size(12.5));
        }
        if let Some((u, removed)) = app.url_result.clone() {
            ui::separator(ui);
            ui.add(egui::Label::new(RichText::new(&u).size(13.0).color(ui::ACCENT)).wrap());
            if removed.is_empty() {
                ui.label(caption("Nothing to remove; this link was already clean."));
            } else {
                ui.label(caption(format!("Removed: {}", removed.join(", "))));
            }
            if ui::soft_button(ui, format!("{} Copy clean link", ph::COPY)).clicked() {
                app.copy_text(ctx, u);
            }
        }
    });
    card(ui, |ui| {
        let mut v = app.store.is_on("url_clean");
        ui::feature_row(ui, ph::MAGIC_WAND, "Clean copied links automatically", "Every link you copy loses its tracking parameters.", |ui| {
            ui::toggle(ui, &mut v);
        });
        if v != app.store.is_on("url_clean") {
            app.store.set_on("url_clean", v);
            app.mark_dirty();
        }
        ui.label(caption("Extra parameters to remove (comma separated)"));
        if ui.add(egui::TextEdit::singleline(&mut app.store.url_extra_params).hint_text("ref, source").desired_width(f32::INFINITY)).changed() {
            app.mark_dirty();
        }
    });
}

pub(crate) fn run_clean(app: &mut ValleApp) {
    match urlclean::clean(&app.url_input, &app.store.url_extra_params) {
        Some(r) => {
            app.url_result = Some((r.url, r.removed));
            app.url_error = None;
        }
        None => {
            app.url_result = None;
            app.url_error = Some("That doesn't look like a single web link.".into());
        }
    }
}
