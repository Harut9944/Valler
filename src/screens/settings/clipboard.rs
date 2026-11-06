//! Clipboard page.

use crate::app::ValleApp;
use crate::screens::tabs::fmt_secs;
use crate::ui::{self, caption, card};
use egui::Ui;
use super::*;

pub(crate) fn clipboard(app: &mut ValleApp, ui: &mut Ui) {
    page_title(ui, "Clipboard", "History stays on this computer and can be cleared anytime.");
    card(ui, |ui| {
        let mut on = app.store.is_on("clipboard");
        setting_row(ui, "Save clipboard history", "Keeps text you copy so you can reuse it from the panel.", |ui| {
            ui::toggle(ui, &mut on);
        });
        if on != app.store.is_on("clipboard") {
            app.store.set_on("clipboard", on);
            app.store.dirty_flag = true;
        }
        ui::separator(ui);
        setting_row(ui, "Items to keep", "Pinned items are always kept.", |ui| {
            let mut n = app.store.clip_max_items as u32;
            if ui.add(egui::DragValue::new(&mut n).range(10..=2000)).changed() {
                app.store.clip_max_items = n as usize;
                app.store.dirty_flag = true;
            }
        });
        ui::separator(ui);
        let mut ac = app.store.is_on("clip_autoclear");
        setting_row(ui, "Clear the clipboard automatically", "Empties the system clipboard a while after you copy. History is kept.", |ui| {
            ui::toggle(ui, &mut ac);
        });
        if ac != app.store.is_on("clip_autoclear") {
            app.store.set_on("clip_autoclear", ac);
            app.store.dirty_flag = true;
        }
        if ac {
            setting_row(ui, "Clear after", "", |ui| {
                let mut secs = app.store.clip_auto_clear_secs.max(5);
                egui::ComboBox::from_id_salt("s-autoclear").selected_text(fmt_secs(secs)).show_ui(ui, |ui| {
                    for s in [15, 30, 60, 120, 300, 900] {
                        ui.selectable_value(&mut secs, s, fmt_secs(s));
                    }
                });
                if secs != app.store.clip_auto_clear_secs {
                    app.store.clip_auto_clear_secs = secs;
                    app.store.dirty_flag = true;
                }
            });
        }
        ui::separator(ui);
        ui.horizontal(|ui| {
            ui.label(caption(format!("{} items saved", app.store.clip_history.len())));
            if ui::soft_button(ui, "Clear history").clicked() {
                app.store.clip_history.clear();
                app.store.dirty_flag = true;
            }
        });
    });
    app.mark_dirty_if_changed();
}
