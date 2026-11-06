//! Keep awake tab.

use crate::app::ValleApp;
use crate::services::keepawake::DURATIONS;
use crate::ui::{self, caption, card, fmt_duration, title};
use egui::{Color32, CornerRadius, RichText, Ui, vec2};
use egui_phosphor::regular as ph;

// ------------------------------------------------------------- keep awake

pub fn keep_awake(app: &mut ValleApp, ui: &mut Ui) {
    let active = app.awake.active() || app.awake.demo_active();
    card(ui, |ui| {
        ui.horizontal(|ui| {
            let color = if active { ui::YELLOW } else { ui::MUTED };
            ui.add_sized([44.0, 44.0], egui::Label::new(ui::icon(if active { ph::SUN } else { ph::MOON_STARS }, 32.0, color)));
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                if active {
                    ui.label(title("Staying awake"));
                    let txt = match app.awake.remaining() {
                        Some(r) => format!("{} left", fmt_duration(r.as_secs() + 59)),
                        None => match &app.awake.auto_reason {
                            Some(r) => format!("Automatically · {r}"),
                            None => "Until you stop it".into(),
                        },
                    };
                    ui.label(caption(txt));
                } else {
                    ui.label(title("Sleep as usual"));
                    ui.label(caption("Your computer follows its normal sleep settings."));
                }
            });
        });
        ui::separator(ui);
        ui.label(RichText::new("Duration").color(ui::MUTED));
        ui.horizontal_wrapped(|ui| {
            for (mins, label) in DURATIONS {
                let sel = app.store.keep_awake_minutes == *mins;
                let b = egui::Button::new(RichText::new(*label).size(13.0).color(if sel { Color32::WHITE } else { ui::TEXT }))
                    .fill(if sel { ui::ACCENT } else { ui::CONTROL })
                    .corner_radius(CornerRadius::same(8));
                if ui.add(b).clicked() {
                    app.store.keep_awake_minutes = *mins;
                    app.mark_dirty();
                    if app.awake.active() {
                        app.awake.start(*mins, app.store.keep_display_on);
                    }
                }
            }
        });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(ui.available_width() - 60.0);
                ui.spacing_mut().item_spacing.y = 1.0;
                ui.label(RichText::new("Keep the display on").size(14.5));
                ui.add(egui::Label::new(caption("Off lets the screen turn off while work keeps running.")).wrap());
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui::toggle(ui, &mut app.store.keep_display_on).changed() {
                    app.mark_dirty();
                    if app.awake.active() {
                        app.awake.start(app.store.keep_awake_minutes, app.store.keep_display_on);
                    }
                }
            });
        });
        ui.add_space(6.0);
        let w = ui.available_width();
        let label = if active { format!("{}  Stop keeping awake", ph::STOP) } else { format!("{}  Keep awake", ph::COFFEE) };
        let b = egui::Button::new(RichText::new(label).font(ui::bold(14.5)).color(Color32::WHITE))
            .fill(if active { Color32::from_rgb(0x5a, 0x2a, 0x2a) } else { ui::ACCENT })
            .corner_radius(CornerRadius::same(10))
            .min_size(vec2(w, 36.0));
        if ui.add(b).clicked() {
            if active {
                app.awake.stop();
            } else {
                app.awake.start(app.store.keep_awake_minutes, app.store.keep_display_on);
            }
        }
        if let Some(e) = &app.awake.error {
            ui.label(RichText::new(e).color(ui::RED).size(12.5));
        }
    });
    card(ui, |ui| {
        ui.label(caption(
            "Valler asks the system itself to stay awake: a power request on Windows, a power assertion on macOS and a session inhibitor on GNOME and KDE. Closing Valler ends it.",
        ));
    });
}
