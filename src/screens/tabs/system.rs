//! System tab: temperatures, processor, graphics, battery and memory.

use crate::app::ValleApp;
use crate::ui::{self, bar, caption, card, fmt_bytes, fmt_duration, sparkline};
use egui::{Color32, CornerRadius, RichText, Sense, Ui, vec2};
use egui_phosphor::regular as ph;
use super::*;

// ------------------------------------------------------------- system

pub fn system(app: &mut ValleApp, ui: &mut Ui) {
    let s = app.snap.clone();
    card(ui, |ui| {
        if !s.ready {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(caption("Reading sensors…"));
            });
        }
        if app.store.is_installed("monitor_temps") {
            ui.label(RichText::new("Temperatures").font(ui::bold(15.0)).color(ui::MUTED));
            let bat_t = s.battery.as_ref().and_then(|b| b.temp_c);
            let w = (ui.available_width() - 16.0) / 3.0;
            ui.horizontal(|ui| {
                temp_tile(ui, w, ph::CPU, "CPU", &temp_text(app, s.cpu_temp));
                temp_tile(ui, w, ph::GRAPHICS_CARD, "GPU", &temp_text(app, s.gpu_temp));
                let (g, l) = if s.battery.is_some() { (ph::BATTERY_HIGH, "Battery") } else { (ph::THERMOMETER, "Hottest") };
                let v = if s.battery.is_some() { bat_t } else { s.temps.iter().map(|t| t.1).fold(None, |a: Option<f32>, t| Some(a.map_or(t, |a| a.max(t)))) };
                temp_tile(ui, w, g, l, &temp_text(app, v));
            });
            if app.expanded.contains("temps") {
                for (l, t) in &s.temps {
                    ui.horizontal(|ui| {
                        ui.label(caption(l));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(temp_text(app, Some(*t))).size(13.0));
                        });
                    });
                }
            }
            if !s.temps.is_empty() {
                let label = if app.expanded.contains("temps") { "Hide sensors" } else { "All sensors" };
                if ui.add(egui::Button::new(RichText::new(format!("{label} ({})", s.temps.len())).size(12.5).color(ui::ACCENT)).frame(false)).clicked()
                    && !app.expanded.remove("temps") {
                        app.expanded.insert("temps");
                    }
            } else if s.ready {
                ui.label(caption("This system does not expose temperature sensors to apps."));
            }
            ui::separator(ui);
        }

        if app.store.is_installed("monitor_cpu") || app.store.is_installed("monitor_gpu") {
            ui.label(RichText::new("Hardware usage").font(ui::bold(15.0)).color(ui::MUTED));
        }
        if app.store.is_installed("monitor_cpu") {
            let open = usage_row(app, ui, "cpu", "CPU", s.cpu_total / 100.0, ui::ACCENT, format!("{:.0}%", s.cpu_total));
            let v: Vec<f32> = s.cpu_hist.iter().copied().collect();
            sparkline(ui, &v, 100.0, ui::ACCENT, 34.0);
            if open {
                ui.label(caption(format!(
                    "{} · {} threads{}",
                    if s.cpu_brand.is_empty() { "Processor" } else { &s.cpu_brand },
                    s.per_core.len(),
                    if s.physical_cores > 0 { format!(" · {} cores", s.physical_cores) } else { String::new() }
                )));
                // Per-core matrix
                let cols = 8usize;
                let w = ui.available_width();
                let cell = (w - (cols as f32 - 1.0) * 4.0) / cols as f32;
                for row in s.per_core.chunks(cols) {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;
                        for u in row {
                            let (r, resp) = ui.allocate_exact_size(vec2(cell, 16.0), Sense::hover());
                            let f = (u / 100.0).clamp(0.0, 1.0);
                            ui.painter().rect_filled(r, CornerRadius::same(3), Color32::from_rgb(0x3a, 0x3a, 0x40));
                            let mut fr = r;
                            fr.set_left(r.right() - r.width() * f);
                            let mut fr2 = r;
                            fr2.set_width(r.width() * f);
                            ui.painter().rect_filled(fr2, CornerRadius::same(3), ui::lerp_color(ui::ACCENT, ui::ORANGE, f));
                            resp.on_hover_text(format!("{u:.0}%"));
                        }
                    });
                }
                if s.load.0 > 0.0 {
                    ui.label(caption(format!("Load average {:.2} · {:.2} · {:.2}", s.load.0, s.load.1, s.load.2)));
                }
            }
        }
        if app.store.is_installed("monitor_gpu") || app.store.is_installed("monitor_cpu") {
            let gpu = s.gpus.iter().filter(|g| g.usage.is_some()).max_by(|a, b| a.usage.partial_cmp(&b.usage).unwrap());
            if let Some(g) = gpu {
                let u = g.usage.unwrap_or(0.0);
                let open = usage_row(app, ui, "gpu", "GPU", u / 100.0, ui::ACCENT, format!("{u:.0}%"));
                let v: Vec<f32> = s.gpu_hist.iter().copied().collect();
                sparkline(ui, &v, 100.0, ui::CYAN, 34.0);
                if open {
                    for g in &s.gpus {
                        let mem = match (g.mem_used, g.mem_total) {
                            (Some(u), Some(t)) if t > 0 => format!(" · {} / {}", fmt_bytes(u), fmt_bytes(t)),
                            _ => String::new(),
                        };
                        ui.label(caption(format!("{}{}", g.name, mem)));
                    }
                }
            }
        }
        if let Some(b) = &s.battery
            && app.store.is_installed("monitor_power") {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(if b.charging { ph::BATTERY_CHARGING } else { ph::BATTERY_HIGH }).color(ui::MUTED));
                    ui.label(RichText::new("Battery").size(15.0));
                    let used: f32 = 110.0 - 60.0;
                    ui.add_space(used.max(4.0));
                    let w = ui.available_width() - 64.0;
                    let col = if b.percent < 20.0 { ui::RED } else { ui::GREEN };
                    bar(ui, w, b.percent / 100.0, col);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(format!("{:.0}%", b.percent)).font(ui::bold(15.0)));
                    });
                });
                let v: Vec<f32> = s.battery_hist.iter().copied().collect();
                sparkline(ui, &v, 100.0, ui::GREEN, 30.0);
            }

        if app.store.is_installed("monitor_cpu") {
            ui::separator(ui);
            let open = chevron_row(app, ui, "busy", "Apps using the most processor");
            if open {
                for p in s.procs.iter().take(6) {
                    ui.horizontal(|ui| {
                        ui.add_space(22.0);
                        ui.add(egui::Label::new(RichText::new(&p.name).size(13.5)).truncate());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(format!("{:.1}%", p.cpu)).size(13.0).color(ui::MUTED));
                        });
                    });
                }
            }
        }

        if app.store.is_installed("monitor_memory") {
            ui::separator(ui);
            ui.label(RichText::new("Memory").font(ui::bold(15.0)).color(ui::MUTED));
            let open = {
                let mut open = false;
                ui.horizontal(|ui| {
                    open = chevron_row(app, ui, "mem", "Pressure");
                    let (txt, col) = s.pressure();
                    ui::status_pill(ui, txt, col);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(format!("{} / {}", fmt_bytes(s.mem_used), fmt_bytes(s.mem_total))).size(14.5).color(ui::MUTED));
                    });
                });
                open
            };
            let v: Vec<f32> = s.mem_hist.iter().copied().collect();
            sparkline(ui, &v, 1.0, ui::CYAN, 30.0);
            if open {
                if s.swap_total > 0 {
                    ui.label(caption(format!("Swap {} of {}", fmt_bytes(s.swap_used), fmt_bytes(s.swap_total))));
                }
                let mut by_mem = s.procs.clone();
                by_mem.sort_by_key(|p| std::cmp::Reverse(p.mem));
                for p in by_mem.iter().take(5) {
                    ui.horizontal(|ui| {
                        ui.add_space(22.0);
                        ui.add(egui::Label::new(RichText::new(&p.name).size(13.5)).truncate());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(fmt_bytes(p.mem)).size(13.0).color(ui::MUTED));
                        });
                    });
                }
            }
        }
        ui::separator(ui);
        ui.horizontal(|ui| {
            ui.label(RichText::new(ph::CLOCK).color(ui::MUTED));
            ui.label(RichText::new(format!("Up for {}", fmt_duration(s.uptime))).size(14.5).color(ui::MUTED));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(caption(&s.host));
            });
        });
    });
}
