//! Displays tab: brightness per screen and software dimming.

use crate::app::ValleApp;
use crate::services::display::{self, Kind};
use crate::ui::{self, caption, card, title};
use egui::{RichText, Ui};
use egui_phosphor::regular as ph;
use std::time::{Duration, Instant};

const READ_EVERY: Duration = Duration::from_secs(10);

pub(crate) fn displays(app: &mut ValleApp, ui: &mut Ui, ctx: &egui::Context) {
    // Re-read now and then: the brightness keys may have changed it.
    if !app.demo && app.displays_read.is_none_or(|t| t.elapsed() > READ_EVERY) {
        app.displays_read = Some(Instant::now());
        display::refresh(app.displays.clone(), ctx.clone());
    }
    let state = app.displays.lock().map(|s| s.clone()).unwrap_or_default();
    match &state.screens {
        None => {
            card(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(caption("Looking for screens…"));
                });
            });
        }
        Some(screens) if screens.is_empty() => {
            card(ui, |ui| {
                ui::empty_note(ui, ph::MONITOR, if cfg!(target_os = "linux") {
                    "No screen with adjustable brightness was found. External monitors need ddcutil and DDC/CI turned on in the monitor's menu. Software dimming below works everywhere."
                } else {
                    "This system doesn't let apps change the screen brightness. Software dimming below works everywhere."
                });
            });
        }
        Some(screens) => {
            for (i, s) in screens.iter().enumerate() {
                card(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(ui::icon(if s.kind == Kind::BuiltIn { ph::LAPTOP } else { ph::MONITOR }, 22.0, ui::ACCENT));
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 0.0;
                            ui.label(title(&s.name));
                            ui.label(caption(if s.kind == Kind::BuiltIn { "Built-in screen" } else { "External monitor · DDC/CI" }));
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(format!("{:.0}%", s.level * 100.0)).font(ui::bold(15.0)));
                        });
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(ph::SUN_DIM).color(ui::MUTED));
                        let mut v = s.level;
                        ui.spacing_mut().slider_width = (ui.available_width() - 30.0).max(60.0);
                        if ui.add(egui::Slider::new(&mut v, 0.0..=1.0).show_value(false).trailing_fill(true)).changed() {
                            if let Some(ctl) = &app.display_ctl {
                                ctl.set(s, v);
                            }
                            if let Ok(mut st) = app.displays.lock()
                                && let Some(list) = st.screens.as_mut()
                            {
                                list[i].level = v;
                            }
                            // Don't let a re-read jump the slider back mid-drag.
                            app.displays_read = Some(Instant::now());
                        }
                        ui.label(RichText::new(ph::SUN).color(ui::MUTED));
                    });
                });
            }
        }
    }
    if let Some(e) = &state.error {
        ui.label(RichText::new(e).size(12.5).color(ui::ORANGE));
    }
    card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(ui::icon(ph::CIRCLE_HALF, 22.0, ui::ACCENT));
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.label(title("Software dimming"));
                ui.add(egui::Label::new(caption("Darkens the screen with a see-through layer, also below its lowest brightness. Clicks pass through.")).wrap());
            });
        });
        ui.horizontal(|ui| {
            let mut v = app.store.dim_level;
            ui.spacing_mut().slider_width = (ui.available_width() - 70.0).max(60.0);
            if ui.add(egui::Slider::new(&mut v, 0.0..=crate::app::MAX_DIM).show_value(false).trailing_fill(true)).changed() {
                app.store.dim_level = v;
                app.mark_dirty();
            }
            ui.label(RichText::new(if v <= 0.0 { "Off".to_string() } else { format!("{:.0}%", v * 100.0) }).font(ui::bold(14.0)));
        });
        if app.store.dim_level > 0.0 && ui::soft_button(ui, "Turn dimming off").clicked() {
            app.store.dim_level = 0.0;
            app.mark_dirty();
        }
    });
}
