//! Power tab: battery charge, health and power use.

use crate::app::ValleApp;
use crate::ui::{self, bar, caption, card, fmt_duration, sparkline, title};
use egui::{RichText, Ui};
use egui_phosphor::regular as ph;
use super::*;

// ------------------------------------------------------------- power

pub fn power(app: &mut ValleApp, ui: &mut Ui) {
    let s = app.snap.clone();
    let Some(b) = s.battery.clone() else {
        card(ui, |ui| {
            ui::empty_note(ui, ph::PLUG, if s.ready { "No battery found. This computer runs on mains power." } else { "Looking for a battery…" });
        });
        card(ui, |ui| {
            grid_row(ui, "Up for", &fmt_duration(s.uptime));
            grid_row(ui, "Processor load", &format!("{:.0}%", s.cpu_total));
        });
        return;
    };
    card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("{:.0}%", b.percent)).font(ui::bold(34.0)));
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.label(title(&b.state));
                let t = if b.charging {
                    b.time_to_full.map(|t| format!("Full in {}", fmt_duration(t)))
                } else {
                    b.time_to_empty.map(|t| format!("{} left", fmt_duration(t)))
                };
                ui.label(caption(t.unwrap_or_else(|| "Estimating time…".into())));
            });
        });
        let col = if b.percent < 20.0 { ui::RED } else { ui::GREEN };
        bar(ui, ui.available_width(), b.percent / 100.0, col);
        let v: Vec<f32> = s.battery_hist.iter().copied().collect();
        sparkline(ui, &v, 100.0, ui::GREEN, 34.0);
    });
    card(ui, |ui| {
        ui.label(RichText::new("Health").font(ui::bold(15.0)).color(ui::MUTED));
        let hc = if b.health >= 80.0 { ui::GREEN } else if b.health >= 60.0 { ui::YELLOW } else { ui::RED };
        ui.horizontal(|ui| {
            ui.label(RichText::new("Capacity").color(ui::MUTED));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(format!("{:.0}%", b.health)).font(ui::bold(14.5)).color(hc));
            });
        });
        if let Some(c) = b.cycles {
            grid_row(ui, "Charge cycles", &c.to_string());
        }
        grid_row(ui, "Holds", &format!("{:.1} Wh of {:.1} Wh new", b.full_wh, b.design_wh));
        grid_row(ui, "Stored now", &format!("{:.1} Wh", b.energy_wh));
        if let Some(t) = b.temp_c {
            grid_row(ui, "Temperature", &temp_text(app, Some(t)));
        }
        if !b.model.is_empty() || !b.vendor.is_empty() {
            grid_row(ui, "Model", format!("{} {}", b.vendor, b.model).trim());
        }
        grid_row(ui, "Chemistry", &b.technology);
    });
    card(ui, |ui| {
        ui.label(RichText::new("Power use").font(ui::bold(15.0)).color(ui::MUTED));
        grid_row(ui, if b.charging { "Charging at" } else { "Drawing" }, &format!("{:.1} W", b.rate_w));
        grid_row(ui, "Voltage", &format!("{:.2} V", b.voltage));
        let v: Vec<f32> = s.power_hist.iter().copied().collect();
        let max = v.iter().copied().fold(5.0, f32::max);
        sparkline(ui, &v, max, ui::ORANGE, 34.0);
    });
}
