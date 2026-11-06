//! Cleaning mode and its full-screen cover.

use crate::app::ValleApp;
use crate::ui::{self, caption, card, title};
use egui::{Color32, RichText, Ui};
use egui_phosphor::regular as ph;
use std::time::{Duration, Instant};

// ------------------------------------------------------------- cleaning mode

pub fn cleaning_view(app: &mut ValleApp, ui: &mut Ui) {
    card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(ui::icon(ph::KEYBOARD, 34.0, ui::ACCENT));
            ui.vertical(|ui| {
                ui.label(title("Clean your keyboard and screen"));
                ui.add(egui::Label::new(caption("Valler covers the screen and ignores typing until you press Esc several times in a row.")).wrap());
            });
        });
        ui::separator(ui);
        ui.horizontal(|ui| {
            ui.label("Presses of Esc to finish");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let mut n = app.store.cleaning_presses;
                if ui.add(egui::DragValue::new(&mut n).range(2..=10)).changed() {
                    app.store.cleaning_presses = n;
                    app.mark_dirty();
                }
            });
        });
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(ui.available_width() - 60.0);
                ui.label("Keep the screen visible");
                ui.add(egui::Label::new(caption("Dims the screen instead of turning it black.")).wrap());
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui::toggle(ui, &mut app.store.cleaning_keep_visible).changed() {
                    app.mark_dirty();
                }
            });
        });
        ui.add_space(6.0);
        let w = ui.available_width();
        let b = egui::Button::new(RichText::new(format!("{}  Start cleaning mode", ph::SPARKLE)).font(ui::bold(14.5)).color(Color32::WHITE))
            .fill(ui::ACCENT)
            .corner_radius(egui::CornerRadius::same(10))
            .min_size(egui::vec2(w, 36.0));
        if ui.add(b).clicked() && !app.demo {
            app.cleaning = true;
            app.cleaning_presses.clear();
        }
    });
    card(ui, |ui| {
        ui.add(egui::Label::new(caption("System shortcuts such as Alt+Tab, the Windows key or Ctrl+Alt+Del still reach the system, so avoid pressing them while wiping.")).wrap());
    });
}

/// The full-screen cover shown while cleaning mode is on.
pub fn cleaning_viewport(app: &mut ValleApp, ctx: &egui::Context) {
    let vid = egui::ViewportId::from_hash_of("valler-cleaning");
    let builder = egui::ViewportBuilder::default()
        .with_title("Valler cleaning mode")
        .with_fullscreen(true)
        .with_decorations(false)
        .with_transparent(app.store.cleaning_keep_visible)
        .with_always_on_top();
    let need = app.store.cleaning_presses.max(2) as usize;
    let visible = app.store.cleaning_keep_visible;
    let mut done = false;
    ctx.show_viewport_immediate(vid, builder, |ui, _| {
        let c = ui.ctx().clone();
        if c.input(|i| i.viewport().close_requested()) {
            c.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
        let events = c.input(|i| i.raw.events.clone());
        for e in events {
            if let egui::Event::Key { key, pressed: true, repeat: false, .. } = e {
                let now = Instant::now();
                if key == egui::Key::Escape {
                    app.cleaning_presses.retain(|t| now.duration_since(*t) < Duration::from_millis(1500));
                    app.cleaning_presses.push(now);
                    if app.cleaning_presses.len() >= need {
                        done = true;
                    }
                } else {
                    app.cleaning_presses.clear();
                }
            }
        }
        let fill = if visible { Color32::from_rgba_unmultiplied(0, 0, 0, 190) } else { Color32::BLACK };
        egui::CentralPanel::default().frame(egui::Frame::new().fill(fill)).show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space((ui.available_height() / 2.0 - 40.0).max(0.0));
                ui.label(ui::icon(ph::SPARKLE, 30.0, Color32::from_gray(90)));
                let left = need.saturating_sub(app.cleaning_presses.len());
                ui.label(RichText::new("Cleaning mode").font(ui::bold(16.0)).color(Color32::from_gray(120)));
                ui.label(RichText::new(format!("Press Esc {left} more time{} quickly to finish", if left == 1 { "" } else { "s" })).size(13.0).color(Color32::from_gray(90)));
            });
        });
        c.request_repaint_after(Duration::from_millis(250));
    });
    if done {
        app.cleaning = false;
        app.cleaning_presses.clear();
        app.toast("Cleaning mode finished");
    }
}
