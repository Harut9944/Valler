//! Snippets page.

use crate::app::ValleApp;
use crate::ui::{self, caption, card};
use egui::{RichText, Ui};
use egui_phosphor::regular as ph;
use super::*;

pub(crate) fn snippets(app: &mut ValleApp, ui: &mut Ui) {
    page_title(ui, "Snippets", "Saved text you can copy in one click from the panel.");
    card(ui, |ui| {
        ui.label(caption("Variables: {{date}}, {{time}}, {{datetime}}, {{clipboard}}, {{name}}. Add a format after a colon, like {{date:%A %d %B}}."));
        ui.label(caption("Typing a trigger anywhere to expand it needs a keyboard hook, which is coming in a later version."));
    });
    let mut remove = None;
    for (i, s) in app.store.snippets.iter_mut().enumerate() {
        card(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.add(egui::TextEdit::singleline(&mut s.trigger).desired_width(160.0)).changed() {
                    app.store.dirty_flag = true;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(RichText::new(ph::TRASH).color(ui::MUTED)).frame(false)).clicked() {
                        remove = Some(i);
                    }
                });
            });
            if ui.add(egui::TextEdit::multiline(&mut s.text).desired_rows(2).desired_width(f32::INFINITY)).changed() {
                app.store.dirty_flag = true;
            }
        });
    }
    if let Some(i) = remove {
        app.store.snippets.remove(i);
        app.store.dirty_flag = true;
    }
    if ui::primary_button(ui, format!("{} Add snippet", ph::PLUS)).clicked() {
        app.store.snippets.push(crate::core::store::Snippet { trigger: ";new".into(), text: String::new() });
        app.store.dirty_flag = true;
    }
    app.mark_dirty_if_changed();
}
