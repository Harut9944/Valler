//! The Shelf page in Utilities.

use crate::app::ValleApp;
use crate::services::shelf::{self};
use crate::ui::{self, caption, card, title};
use egui::Ui;
use egui_phosphor::regular as ph;
use std::sync::{Arc, Mutex};
use super::*;

pub type ThumbQueue = Arc<Mutex<Vec<(u64, egui::ColorImage)>>>;

// ------------------------------------------------------------- full page

pub fn shelf_view(app: &mut ValleApp, ui: &mut Ui, ctx: &egui::Context) {
    if app.demo && app.store.shelf_items.is_empty() {
        app.store.shelf_items = shelf::demo();
        app.shelf_selected.insert(2);
    }
    handle_paste_key(app, ctx);
    card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(ui::icon(ph::TRAY, 22.0, ui::ACCENT));
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.label(title("Shelf"));
                ui.label(caption(shelf::summary(&app.store.shelf_items)));
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let label = if app.shelf_floating { "Hide floating" } else { "Float" };
                if ui::soft_button(ui, format!("{} {label}", ph::PICTURE_IN_PICTURE)).on_hover_text("A small Shelf window that stays on top of other apps").clicked() && !app.demo {
                    app.shelf_floating = !app.shelf_floating;
                }
            });
        });
        ui.horizontal(|ui| {
            if ui::soft_button(ui, format!("{} Add files…", ph::PLUS)).clicked() && !app.demo {
                do_add_files(app);
            }
            if ui::soft_button(ui, format!("{} Paste", ph::CLIPBOARD)).on_hover_text("Adds files you copied in a file manager, or copied text and links").clicked() && !app.demo {
                do_paste(app);
            }
        });
        ui.horizontal(|ui| {
            let r = ui.add(egui::TextEdit::singleline(&mut app.shelf_note).hint_text("Type a note or link and press Enter").desired_width(f32::INFINITY));
            if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !app.shelf_note.trim().is_empty() {
                let t = std::mem::take(&mut app.shelf_note);
                shelf::add_text(&mut app.store.shelf_items, &t);
                app.mark_dirty();
            }
        });
    });
    card(ui, |ui| {
        tiles(app, ui, ctx, 96.0);
        if !app.store.shelf_items.is_empty() {
            ui::separator(ui);
            action_bar(app, ui);
            ui.label(caption("Click to select, Ctrl+click for more, double-click to open, right-click for more actions."));
        }
    });
    card(ui, |ui| {
        let mut v = app.store.shelf_remove_after_use;
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(ui.available_width() - 60.0);
                ui.label("Take items off after copying or moving them");
                ui.add(egui::Label::new(caption("Pinned items always stay.")).wrap());
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui::toggle(ui, &mut v);
            });
        });
        if v != app.store.shelf_remove_after_use {
            app.store.shelf_remove_after_use = v;
            app.mark_dirty();
        }
        if cfg!(target_os = "linux") {
            ui::separator(ui);
            let mut x = app.store.linux_x11;
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width() - 60.0);
                    ui.label("Accept dragged files on Wayland");
                    ui.add(egui::Label::new(caption("Runs Valler through XWayland so files can be dropped on it. Takes effect after restarting Valler.")).wrap());
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui::toggle(ui, &mut x);
                });
            });
            if x != app.store.linux_x11 {
                app.store.linux_x11 = x;
                app.mark_dirty();
            }
        }
    });
}
