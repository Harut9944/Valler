//! Monitor page: this computer, units, speed test file and alerts.

use crate::app::ValleApp;
use crate::ui::{self, caption, card, title};
use egui::{RichText, Ui};
use egui_phosphor::regular as ph;
use super::*;

pub(crate) fn monitor(app: &mut ValleApp, ui: &mut Ui) {
    page_title(ui, "Monitor", "System readings in the panel.");
    let s = app.snap.clone();
    card(ui, |ui| {
        ui.label(title("This computer"));
        for (k, v) in [
            ("Name", s.host.clone()),
            ("System", s.os.clone()),
            ("Architecture", s.arch.clone()),
            ("Processor", s.cpu_brand.clone()),
            ("Threads", s.per_core.len().to_string()),
            ("Memory", ui::fmt_bytes(s.mem_total)),
            ("Sensors found", s.temps.len().to_string()),
            ("Graphics", s.gpus.iter().map(|g| g.name.clone()).collect::<Vec<_>>().join(", ")),
            ("Battery", s.battery.as_ref().map(|b| format!("{} {}", b.vendor, b.model)).unwrap_or_else(|| "None".into())),
        ] {
            ui.horizontal(|ui| {
                ui.label(RichText::new(k).color(ui::MUTED));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(if v.trim().is_empty() { "—".to_string() } else { v });
                });
            });
        }
    });
    card(ui, |ui| {
        setting_row(ui, "Temperatures in Fahrenheit", "", |ui| {
            if ui::toggle(ui, &mut app.store.temperature_fahrenheit).changed() {
                app.store.dirty_flag = true;
            }
        });
        ui::separator(ui);
        setting_row(ui, "Speed test file", "Downloaded only when you run the test.", |ui| {
            if ui.add(egui::TextEdit::singleline(&mut app.store.speedtest_url).desired_width(320.0)).changed() {
                app.store.dirty_flag = true;
            }
        });
    });
    if app.store.is_installed("monitor_alerts") {
        alerts_card(app, ui);
    }
    app.mark_dirty_if_changed();
}

/// One alert: a switch, and the limit it watches (if it has one).
pub(crate) fn alert_row(ui: &mut Ui, on: &mut bool, name: &str, desc: &str, limit: Option<(&mut u32, std::ops::RangeInclusive<u32>, &str)>) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_width((ui.available_width() - 230.0).max(220.0));
            ui.spacing_mut().item_spacing.y = 1.0;
            ui.label(RichText::new(name).size(14.5));
            ui.add(egui::Label::new(caption(desc)).wrap());
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            changed |= ui::toggle(ui, on).changed();
            if let Some((v, range, suffix)) = limit {
                ui.add_space(10.0);
                ui.add_enabled_ui(*on, |ui| {
                    changed |= ui.add(egui::DragValue::new(v).range(range).suffix(suffix)).changed();
                });
            }
        });
    });
    changed
}

pub(crate) fn alerts_card(app: &mut ValleApp, ui: &mut Ui) {
    let deg = if app.store.temperature_fahrenheit { " °F" } else { " °C" };
    card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(title(format!("{}  Alerts", ph::BELL_RINGING)));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui::soft_button(ui, "Send a test notification").clicked() {
                    crate::services::alerts::notify("Valler alerts are working", "You'll see notifications like this when a limit is crossed.");
                }
            });
        });
        ui.add(egui::Label::new(caption("Desktop notifications when something needs attention. A reading has to stay above its limit for 12 seconds, so short bursts are ignored.")).wrap());
        ui::separator(ui);
        let a = &mut app.store.alerts;
        let mut c = false;
        c |= alert_row(ui, &mut a.cpu, "Processor load", "Sustained processor use above the limit.", Some((&mut a.cpu_threshold, 50..=100, " %")));
        // Temperatures are stored in °C; the field shows °C either way to keep the limit exact.
        let _ = deg;
        c |= alert_row(ui, &mut a.cpu_temp, "Processor temperature", "Sustained processor temperature above the limit (°C).", Some((&mut a.cpu_temp_threshold, 50..=110, " °C")));
        c |= alert_row(ui, &mut a.memory, "Memory pressure", "Memory is almost full and the system starts swapping heavily.", None);
        c |= alert_row(ui, &mut a.disk, "Low disk space", "A drive has less free space than the limit.", Some((&mut a.disk_free_percent, 1..=50, " % free")));
        c |= alert_row(ui, &mut a.battery, "Low battery", "On battery and the charge drops to the limit.", Some((&mut a.battery_percent, 5..=50, " %")));
        c |= alert_row(ui, &mut a.battery_temp, "Battery temperature", "Sustained battery temperature above the limit (°C).", Some((&mut a.battery_temp_threshold, 30..=60, " °C")));
        ui::separator(ui);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Wait between repeats").size(14.5));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                c |= ui.add(egui::DragValue::new(&mut a.cooldown_minutes).range(1..=240).suffix(" min")).changed();
            });
        });
        if c {
            app.store.dirty_flag = true;
        }
        if let Some(m) = &app.alerts.last_message {
            ui.label(RichText::new(format!("Last alert: {m}")).size(12.0).color(ui::FAINT));
        }
    });
}
