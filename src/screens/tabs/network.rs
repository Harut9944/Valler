//! Network tab: live traffic, interfaces and the speed test.

use crate::app::ValleApp;
use crate::services::speedtest::{self, SpeedState};
use crate::ui::{self, caption, card, fmt_bytes, fmt_rate, sparkline, title};
use egui::{RichText, Ui, vec2};
use egui_phosphor::regular as ph;
use super::*;

// ------------------------------------------------------------- network

pub fn network(app: &mut ValleApp, ui: &mut Ui, ctx: &egui::Context) {
    let s = app.snap.clone();
    card(ui, |ui| {
        ui.horizontal(|ui| {
            let w = (ui.available_width() - 8.0) / 2.0;
            for (glyph, label, rate, col) in [(ph::ARROW_DOWN, "Download", s.rx_rate, ui::GREEN), (ph::ARROW_UP, "Upload", s.tx_rate, ui::ACCENT)] {
                ui.allocate_ui(vec2(w, 54.0), |ui| {
                    ui.set_min_width(w);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        ui.label(RichText::new(format!("{glyph} {label}")).color(col).size(13.0));
                        ui.label(RichText::new(fmt_rate(rate)).font(ui::bold(21.0)));
                    });
                });
            }
        });
        let rx: Vec<f32> = s.rx_hist.iter().copied().collect();
        let tx: Vec<f32> = s.tx_hist.iter().copied().collect();
        let max = rx.iter().chain(tx.iter()).copied().fold(1024.0f32, f32::max);
        let top = ui.cursor().min;
        sparkline(ui, &rx, max, ui::GREEN, 46.0);
        let after = ui.cursor().min;
        // Overlay upload on the same area.
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(top, vec2(ui.available_width(), 46.0))));
        sparkline(&mut child, &tx, max, ui::ACCENT, 46.0);
        let _ = after;
        ui::separator(ui);
        grid_row(ui, "This session", &format!("{} {}   {} {}", ph::ARROW_DOWN, fmt_bytes(s.session_rx), ph::ARROW_UP, fmt_bytes(s.session_tx)));
        let ip = s.local_ip.clone().unwrap_or_else(|| "Not connected".into());
        ui.horizontal(|ui| {
            ui.label(RichText::new("Local IP").color(ui::MUTED));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if s.local_ip.is_some() && ui.add(egui::Button::new(RichText::new(ph::COPY).color(ui::MUTED)).frame(false)).on_hover_text("Copy").clicked() {
                    app.copy_text(ctx, ip.clone());
                }
                ui.label(RichText::new(&ip).font(ui::bold(14.0)));
            });
        });
        grid_row(ui, "Computer name", &s.host);
    });
    card(ui, |ui| {
        let open = chevron_row(app, ui, "ifaces", &format!("Interfaces ({})", s.ifaces.len()));
        if open {
            for i in &s.ifaces {
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&i.name).font(ui::bold(14.0)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(caption(format!("{} {}  {} {}", ph::ARROW_DOWN, fmt_rate(i.rx_rate), ph::ARROW_UP, fmt_rate(i.tx_rate))));
                    });
                });
                let ips: Vec<&String> = i.ips.iter().filter(|ip| !ip.starts_with("fe80")).collect();
                if !ips.is_empty() {
                    ui.label(caption(ips.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")));
                }
                ui.label(caption(format!("Total {} received · {} sent", fmt_bytes(i.total_rx), fmt_bytes(i.total_tx))));
            }
        }
    });
    card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(title(format!("{}  Speed test", ph::GAUGE)));
        });
        let st = app.speed.lock().map(|s| s.clone()).unwrap_or_default();
        match &st {
            SpeedState::Idle => {
                ui.label(caption("Downloads a test file to measure your connection. Only runs when you press the button."));
            }
            SpeedState::Running { bytes, secs } => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    let mbps = if *secs > 0.0 { *bytes as f64 * 8.0 / secs / 1e6 } else { 0.0 };
                    ui.label(RichText::new(format!("{mbps:.1} Mbit/s")).font(ui::bold(18.0)));
                    ui.label(caption(fmt_bytes(*bytes)));
                });
            }
            SpeedState::Done { mbps, latency_ms } => {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("{mbps:.1}")).font(ui::bold(26.0)).color(ui::GREEN));
                    ui.label(RichText::new("Mbit/s down").color(ui::MUTED));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(caption(format!("First byte {latency_ms:.0} ms")));
                    });
                });
            }
            SpeedState::Failed(e) => {
                ui.label(RichText::new(format!("The test failed: {e}")).color(ui::RED).size(12.5));
            }
        }
        let running = matches!(st, SpeedState::Running { .. });
        ui.add_enabled_ui(!running, |ui| {
            if ui::primary_button(ui, if matches!(st, SpeedState::Idle) { "Run speed test" } else { "Run again" }).clicked() {
                speedtest::start(app.speed.clone(), app.store.speedtest_url.clone(), ctx.clone());
            }
        });
    });
}
