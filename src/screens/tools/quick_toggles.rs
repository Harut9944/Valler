//! Quick toggles.

use crate::app::ValleApp;
use crate::services::quick_toggles;
use crate::ui::{self, caption, card, title};
use egui::{Color32, RichText, Ui};
use egui_phosphor::regular as ph;
use std::time::{Duration, Instant};

// ------------------------------------------------------------- quick toggles

pub(crate) fn toggle_tile(ui: &mut Ui, w: f32, glyph: &str, title_text: &str, caption_text: &str, active: bool) -> egui::Response {
    let id = ui.next_auto_id();
    let hovered = ui.ctx().read_response(id).is_some_and(|r| r.hovered());
    let inner = egui::Frame::new()
        .fill(if active { ui::ACCENT_SOFT } else if hovered { ui::CARD_HOVER } else { ui::CARD })
        .stroke(egui::Stroke::new(1.0, if active { ui::ACCENT } else { ui::BORDER }))
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(w - 24.0);
            ui.set_min_height(84.0);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 3.0;
                ui.label(ui::icon(glyph, 24.0, if active { ui::ACCENT } else { ui::TEXT }));
                ui.label(RichText::new(title_text).font(ui::bold(14.0)));
                ui.add(egui::Label::new(RichText::new(caption_text).size(11.5).color(ui::MUTED)).wrap());
            });
        });
    ui.interact(inner.response.rect, id, egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn quick_toggles_view(app: &mut ValleApp, ui: &mut Ui, ctx: &egui::Context) {
    if app.demo {
        if app.qt_checked.is_none() {
            app.qt_checked = Some(Instant::now());
            app.qt_dark = Some(true);
            app.qt_trash = Some(12);
        }
    } else if app.qt_checked.is_none_or(|t| t.elapsed() > Duration::from_secs(4)) {
        app.qt_checked = Some(Instant::now());
        app.qt_dark = quick_toggles::dark_mode();
        app.qt_trash = quick_toggles::trash_count();
    }
    let w = (ui.available_width() - 10.0) / 2.0;
    let dark = app.qt_dark.unwrap_or(false);
    let mut action: Option<u8> = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 10.0;
        let (t, c) = if dark { ("Switch to light", "The desktop uses a dark look") } else { ("Switch to dark", "The desktop uses a light look") };
        if toggle_tile(ui, w, if dark { ph::MOON } else { ph::SUN }, t, c, dark).clicked() {
            action = Some(0);
        }
        if toggle_tile(ui, w, ph::LOCK, "Lock screen", "Locks the computer right away", false).clicked() {
            action = Some(1);
        }
    });
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 10.0;
        if toggle_tile(ui, w, ph::MONITOR, "Display off", "Until you move the mouse or press a key", false).clicked() {
            action = Some(2);
        }
        if toggle_tile(ui, w, ph::SPARKLE, "Screen saver", "Starts the screen saver now", false).clicked() {
            action = Some(3);
        }
    });
    let trash_caption = match app.qt_trash {
        Some(0) => "The trash is already empty".to_string(),
        Some(n) => format!("{n} item{} in the trash", if n == 1 { "" } else { "s" }),
        None => "Deletes what is in the trash for good".into(),
    };
    ui.horizontal(|ui| {
        if toggle_tile(ui, w, ph::TRASH, "Empty trash", &trash_caption, false).clicked() && app.qt_trash != Some(0) {
            app.qt_confirm_trash = true;
        }
    });
    if app.qt_confirm_trash {
        card(ui, |ui| {
            ui.label(title("Empty the trash?"));
            ui.label(caption("Everything in the trash is deleted for good. This can't be undone."));
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new(RichText::new("Empty trash").color(Color32::WHITE)).fill(ui::RED)).clicked() {
                    action = Some(4);
                    app.qt_confirm_trash = false;
                }
                if ui::soft_button(ui, "Cancel").clicked() {
                    app.qt_confirm_trash = false;
                }
            });
        });
    }
    let Some(a) = action else { return };
    if app.demo {
        return;
    }
    let out = app.qt_outcome.clone();
    let ctx = ctx.clone();
    match a {
        0 => {
            app.qt_dark = Some(!dark);
            quick_toggles::spawn(out, ctx, move || match quick_toggles::set_dark_mode(!dark) {
                Ok(_) if dark => "Switched to light mode".into(),
                Ok(_) => "Switched to dark mode".into(),
                Err(e) => e,
            });
        }
        1 => quick_toggles::spawn(out, ctx, || quick_toggles::lock_screen().map(|_| "Screen locked".to_string()).unwrap_or_else(|e| e)),
        2 => quick_toggles::spawn(out, ctx, || quick_toggles::display_off().map(|_| "Display off".to_string()).unwrap_or_else(|e| e)),
        3 => quick_toggles::spawn(out, ctx, || quick_toggles::screen_saver().map(|_| "Screen saver started".to_string()).unwrap_or_else(|e| e)),
        _ => quick_toggles::spawn(out, ctx, || match quick_toggles::empty_trash() {
            Ok(n) if n > 0 => format!("Deleted {n} items from the trash"),
            Ok(_) => "Trash emptied".into(),
            Err(e) => format!("Couldn't empty the trash: {e}"),
        }),
    }
}
