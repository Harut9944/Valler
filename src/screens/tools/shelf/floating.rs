//! The floating, always-on-top Shelf window.

use crate::app::ValleApp;
use crate::services::shelf::{self};
use crate::ui::{self, caption};
use egui::{RichText, Ui, pos2, vec2};
use egui_phosphor::regular as ph;
use super::*;

// ------------------------------------------------------------- floating shelf

pub fn floating(app: &mut ValleApp, ctx: &egui::Context) {
    let vid = egui::ViewportId::from_hash_of("valler-shelf");
    let builder = egui::ViewportBuilder::default()
        .with_title("Valler Shelf")
        .with_app_id("valler")
        .with_inner_size([372.0, 330.0])
        .with_min_inner_size([240.0, 200.0])
        .with_decorations(false)
        .with_transparent(true)
        .with_always_on_top();
    ctx.show_viewport_immediate(vid, builder, |ui, _| {
        let c = ui.ctx().clone();
        if c.input(|i| i.viewport().close_requested()) {
            app.shelf_floating = false;
        }
        floating_contents(app, ui, &c);
    });
}

/// Body of the floating Shelf (also drawn inline in screenshot mode).
pub fn floating_contents(app: &mut ValleApp, ui: &mut Ui, c: &egui::Context) {
    let c = c.clone();
    {
        if app.demo && app.store.shelf_items.is_empty() {
            app.store.shelf_items = shelf::demo();
        }
        ui::window_surface(ui);
        handle_paste_key(app, &c);
        if c.input(|i| !i.raw.hovered_files.is_empty()) {
            drop_overlay(&c, "Drop to keep it here");
        }
        take_drops(app, &c);
        let mut close = false;
        egui::Panel::top("shelf-head")
            .show_separator_line(false)
            .frame(egui::Frame::new().inner_margin(egui::Margin { left: 14, right: 14, top: 10, bottom: 4 }))
            .show(ui, |ui| {
                let head = ui.max_rect().expand2(vec2(14.0, 10.0));
                ui::drag_area(ui, egui::Rect::from_min_max(head.min, pos2(head.max.x, head.min.y + 40.0)), "shelf-drag", false);
                ui.horizontal(|ui| {
                    ui.label(ui::icon(ph::TRAY, 17.0, ui::ACCENT));
                    ui.label(RichText::new("Shelf").font(ui::bold(14.0)));
                    ui.label(caption(shelf::summary(&app.store.shelf_items)));
                });
                let x = pos2(head.right() - 20.0, head.top() + 18.0);
                if ui::chrome_button(ui, x, ph::X, "shelf-close", true).on_hover_text("Hide the floating Shelf").clicked() {
                    close = true;
                }
                if ui::chrome_button(ui, x - vec2(30.0, 0.0), ph::PLUS, "shelf-add", false).on_hover_text("Add files…").clicked() {
                    do_add_files(app);
                }
                if ui::chrome_button(ui, x - vec2(60.0, 0.0), ph::CLIPBOARD, "shelf-paste", false).on_hover_text("Paste copied files or text").clicked() {
                    do_paste(app);
                }
            });
        egui::Panel::bottom("shelf-foot")
            .show_separator_line(false)
            .frame(egui::Frame::new().inner_margin(egui::Margin { left: 14, right: 14, top: 4, bottom: 12 }))
            .show(ui, |ui| action_bar(app, ui));
        egui::CentralPanel::default()
            .frame(egui::Frame::new().inner_margin(egui::Margin::symmetric(14, 4)))
            .show(ui, |ui| {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| tiles(app, ui, &c, 80.0));
            });
        ui::resize_grip(ui, "shelf-grip");
        if close {
            app.shelf_floating = false;
        }
    }
}
