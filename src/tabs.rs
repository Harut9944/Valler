//! Panel tab pages: Keep awake, Volume mixer, System, Network, Disks, Power
//! and Controls.

use crate::app::ValleApp;
use crate::features::{self, Group};
use crate::keepawake_ctl::DURATIONS;
use crate::mixer;
use crate::speedtest::{self, SpeedState};
use crate::ui::{self, bar, caption, card, fmt_bytes, fmt_duration, fmt_rate, sparkline, title};
use egui::{Color32, CornerRadius, RichText, Sense, Ui, vec2};
use egui_phosphor::regular as ph;
use std::time::Instant;

fn chevron_row(app: &mut ValleApp, ui: &mut Ui, key: &'static str, label: &str) -> bool {
    let open = app.expanded.contains(key);
    let r = ui
        .horizontal(|ui| {
            ui.label(RichText::new(if open { ph::CARET_DOWN } else { ph::CARET_RIGHT }).color(ui::MUTED));
            ui.label(RichText::new(label).size(15.0).color(ui::TEXT));
        })
        .response;
    let r = ui.interact(r.rect, ui.id().with(key), Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
    if r.clicked() {
        if open {
            app.expanded.remove(key);
        } else {
            app.expanded.insert(key);
        }
    }
    open
}

fn usage_row(app: &mut ValleApp, ui: &mut Ui, key: &'static str, label: &str, frac: f32, color: Color32, value: String) -> bool {
    let mut open = false;
    ui.horizontal(|ui| {
        let left = ui.cursor().min.x;
        open = chevron_row(app, ui, key, label);
        let used = ui.cursor().min.x - left;
        let pad = (110.0 - used).max(4.0);
        ui.add_space(pad);
        let w = ui.available_width() - 64.0;
        bar(ui, w, frac, color);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(value).font(ui::bold(15.0)));
        });
    });
    open
}

fn temp_text(app: &ValleApp, c: Option<f32>) -> String {
    match c {
        Some(c) if app.store.temperature_fahrenheit => format!("{:.0} °F", c * 9.0 / 5.0 + 32.0),
        Some(c) => format!("{c:.0} °C"),
        None => "—".into(),
    }
}

fn temp_tile(ui: &mut Ui, w: f32, glyph: &str, label: &str, value: &str) {
    egui::Frame::new()
        .fill(Color32::from_rgb(0x31, 0x31, 0x35))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(egui::Margin::symmetric(6, 8))
        .show(ui, |ui| {
            ui.set_width(w - 12.0);
            ui.vertical_centered(|ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                ui.label(RichText::new(format!("{glyph} {label}")).size(13.0).color(ui::MUTED));
                ui.label(RichText::new(value).font(ui::bold(21.0)));
            });
        });
}

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
                        None => "Until you stop it".into(),
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

// ------------------------------------------------------------- mixer

fn app_badge(ui: &mut Ui, name: &str) {
    let (r, _) = ui.allocate_exact_size(vec2(38.0, 38.0), Sense::hover());
    let hash = name.bytes().fold(7u32, |h, b| h.wrapping_mul(31).wrapping_add(b as u32));
    let palette = [ui::ACCENT, ui::PURPLE, ui::ORANGE, ui::GREEN, ui::CYAN, Color32::from_rgb(0xff, 0x37, 0x5f)];
    let c = palette[(hash as usize) % palette.len()];
    ui.painter().rect_filled(r, CornerRadius::same(9), c);
    let letter = name.chars().next().unwrap_or('?').to_uppercase().to_string();
    ui.painter().text(r.center(), egui::Align2::CENTER_CENTER, letter, ui::bold(18.0), Color32::WHITE);
}

pub fn mixer(app: &mut ValleApp, ui: &mut Ui) {
    let state = app.mixer.lock().map(|m| m.clone()).unwrap_or_default();
    if !state.available {
        card(ui, |ui| {
            ui::empty_note(ui, ph::SPEAKER_SIMPLE_SLASH, state.message.as_deref().unwrap_or("Checking the sound system…"));
        });
        return;
    }
    let max = if app.store.mixer_boost { 2.0 } else { 1.0 };
    if app.store.is_installed("mixer") {
        card(ui, |ui| {
            if state.streams.is_empty() {
                ui::empty_note(ui, ph::SPEAKER_HIGH, "No app is playing sound right now.");
            }
            for (i, s) in state.streams.iter().enumerate() {
                if i > 0 {
                    ui.add_space(8.0);
                }
                ui.horizontal(|ui| {
                    app_badge(ui, &s.app);
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing.y = 0.0;
                                ui.label(RichText::new(&s.app).font(ui::bold(15.0)));
                                if !s.media.is_empty() {
                                    ui.add(egui::Label::new(caption(&s.media)).truncate());
                                }
                            });
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let cur = state.sinks.iter().find(|d| d.id == s.sink).map(|d| d.description.clone()).unwrap_or_else(|| "Default".into());
                                egui::ComboBox::from_id_salt(("sink", s.id))
                                    .selected_text(short(&cur, 16))
                                    .width(130.0)
                                    .show_ui(ui, |ui| {
                                        for d in &state.sinks {
                                            if ui.selectable_label(d.id == s.sink, &d.description).clicked() {
                                                let (sid, did) = (s.id, d.id);
                                                mixer::spawn_cmd(move || mixer::move_stream(sid, did));
                                            }
                                        }
                                    });
                            });
                        });
                        ui.horizontal(|ui| {
                            let mut v = s.volume;
                            let boosted = v > 1.0;
                            let col = if boosted { ui::ORANGE } else { ui::ACCENT };
                            ui.visuals_mut().selection.bg_fill = col;
                            ui.spacing_mut().slider_width = (ui.available_width() - 120.0).max(60.0);
                            let r = ui.add(egui::Slider::new(&mut v, 0.0..=max).show_value(false).trailing_fill(true));
                            if r.changed() {
                                app.mixer_hold = Some(Instant::now());
                                if let Ok(mut m) = app.mixer.lock() {
                                    if let Some(st) = m.streams.iter_mut().find(|x| x.id == s.id) {
                                        st.volume = v;
                                    }
                                }
                                let id = s.id;
                                mixer::spawn_cmd(move || mixer::set_stream_volume(id, v));
                            }
                            let pct = format!("{}{:.0}%", if boosted { ph::LIGHTNING } else { "" }, v * 100.0);
                            ui.label(RichText::new(pct).font(ui::bold(14.0)).color(if boosted { ui::ORANGE } else { ui::MUTED }));
                            if (v - 1.0).abs() > 0.01
                                && ui.add(egui::Button::new(RichText::new(ph::ARROW_COUNTER_CLOCKWISE).color(if boosted { ui::ORANGE } else { ui::MUTED })).frame(false)).on_hover_text("Back to 100%").clicked()
                            {
                                let id = s.id;
                                mixer::spawn_cmd(move || mixer::set_stream_volume(id, 1.0));
                                app.mixer_hold = None;
                                app.mixer_last = None;
                            }
                            let glyph = if s.muted { ph::SPEAKER_SLASH } else { ph::SPEAKER_HIGH };
                            if ui.add(egui::Button::new(RichText::new(glyph).color(if s.muted { ui::RED } else { ui::MUTED })).frame(false)).on_hover_text("Mute").clicked() {
                                let (id, m) = (s.id, !s.muted);
                                mixer::spawn_cmd(move || mixer::set_stream_mute(id, m));
                                app.mixer_last = None;
                            }
                        });
                    });
                });
            }
        });

        // Output device
        card(ui, |ui| {
            ui.label(title(format!("{}  Output", ph::SPEAKER_HIGH)));
            device_controls(app, ui, &state, true);
        });
    }
    if app.store.is_installed("mic_mute") && !state.sources.is_empty() {
        card(ui, |ui| {
            ui.label(title(format!("{}  Microphone", ph::MICROPHONE)));
            device_controls(app, ui, &state, false);
        });
    }
}

fn device_controls(app: &mut ValleApp, ui: &mut Ui, state: &mixer::MixerState, output: bool) {
    let (list, def_name) = if output { (&state.sinks, &state.default_sink) } else { (&state.sources, &state.default_source) };
    let current = list.iter().find(|d| &d.name == def_name);
    let cur_desc = current.map(|d| d.description.clone()).unwrap_or_else(|| "Default".into());
    egui::ComboBox::from_id_salt(("dev", output))
        .selected_text(short(&cur_desc, 38))
        .width(ui.available_width())
        .show_ui(ui, |ui| {
            for d in list {
                if ui.selectable_label(&d.name == def_name, &d.description).clicked() {
                    let n = d.name.clone();
                    if output {
                        mixer::spawn_cmd(move || mixer::set_default_sink(&n));
                    } else {
                        mixer::spawn_cmd(move || mixer::set_default_source(&n));
                    }
                    app.mixer_last = None;
                }
            }
        });
    if let Some(d) = current {
        ui.horizontal(|ui| {
            let mut v = d.volume;
            ui.spacing_mut().slider_width = (ui.available_width() - 90.0).max(60.0);
            if ui.add(egui::Slider::new(&mut v, 0.0..=1.0).show_value(false).trailing_fill(true)).changed() {
                app.mixer_hold = Some(Instant::now());
                if let Ok(mut m) = app.mixer.lock() {
                    let l = if output { &mut m.sinks } else { &mut m.sources };
                    if let Some(x) = l.iter_mut().find(|x| x.name == d.name) {
                        x.volume = v;
                    }
                }
                let n = d.name.clone();
                if output {
                    mixer::spawn_cmd(move || mixer::set_sink_volume(&n, v));
                } else {
                    mixer::spawn_cmd(move || mixer::set_source_volume(&n, v));
                }
            }
            ui.label(RichText::new(format!("{:.0}%", v * 100.0)).color(ui::MUTED));
            let glyph = match (output, d.muted) {
                (true, false) => ph::SPEAKER_HIGH,
                (true, true) => ph::SPEAKER_SLASH,
                (false, false) => ph::MICROPHONE,
                (false, true) => ph::MICROPHONE_SLASH,
            };
            let tip = if d.muted { "Unmute" } else { "Mute" };
            if ui.add(egui::Button::new(RichText::new(glyph).size(17.0).color(if d.muted { ui::RED } else { ui::TEXT })).frame(false)).on_hover_text(tip).clicked() {
                let (n, m) = (d.name.clone(), !d.muted);
                if output {
                    mixer::spawn_cmd(move || mixer::set_sink_mute(&n, m));
                } else {
                    mixer::spawn_cmd(move || mixer::set_source_mute(&n, m));
                }
                app.mixer_last = None;
            }
        });
        if !output && d.muted {
            ui.label(caption("The microphone is muted for all apps that use this input."));
        }
    }
}

fn short(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(n - 1).collect::<String>())
    }
}

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
                if ui.add(egui::Button::new(RichText::new(format!("{label} ({})", s.temps.len())).size(12.5).color(ui::ACCENT)).frame(false)).clicked() {
                    if !app.expanded.remove("temps") {
                        app.expanded.insert("temps");
                    }
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
        if let Some(b) = &s.battery {
            if app.store.is_installed("monitor_power") {
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
                by_mem.sort_by(|a, b| b.mem.cmp(&a.mem));
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

fn grid_row(ui: &mut Ui, k: &str, v: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(k).color(ui::MUTED));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(v).size(14.0));
        });
    });
}

// ------------------------------------------------------------- disks

pub fn disk(app: &mut ValleApp, ui: &mut Ui) {
    let s = app.snap.clone();
    card(ui, |ui| {
        ui.label(RichText::new("Activity").font(ui::bold(15.0)).color(ui::MUTED));
        ui.horizontal(|ui| {
            let w = (ui.available_width() - 8.0) / 2.0;
            for (label, rate, col) in [("Read", s.disk_read_rate, ui::CYAN), ("Write", s.disk_write_rate, ui::ORANGE)] {
                ui.allocate_ui(vec2(w, 48.0), |ui| {
                    ui.set_min_width(w);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        ui.label(RichText::new(label).color(col).size(13.0));
                        ui.label(RichText::new(fmt_rate(rate)).font(ui::bold(20.0)));
                    });
                });
            }
        });
        let r: Vec<f32> = s.disk_read_hist.iter().copied().collect();
        let w: Vec<f32> = s.disk_write_hist.iter().copied().collect();
        let max = r.iter().chain(w.iter()).copied().fold(64.0 * 1024.0, f32::max);
        let top = ui.cursor().min;
        sparkline(ui, &r, max, ui::CYAN, 40.0);
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(top, vec2(ui.available_width(), 40.0))));
        sparkline(&mut child, &w, max, ui::ORANGE, 40.0);
    });
    if s.disks.is_empty() && s.ready {
        card(ui, |ui| ui::empty_note(ui, ph::HARD_DRIVES, "No drives were reported by the system."));
    }
    for d in &s.disks {
        card(ui, |ui| {
            let used = d.total.saturating_sub(d.avail);
            let frac = used as f32 / d.total.max(1) as f32;
            ui.horizontal(|ui| {
                ui.label(ui::icon(if d.removable { ph::USB } else { ph::HARD_DRIVE }, 24.0, ui::ACCENT));
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    let name = if d.mount == "/" { "System drive".to_string() } else { d.mount.clone() };
                    ui.label(title(name));
                    ui.label(caption(format!("{} · {}{}", d.name, d.fs, if d.kind == "Unknown(-1)" || d.kind.starts_with("Unknown") { String::new() } else { format!(" · {}", d.kind) })));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("{:.0}%", frac * 100.0)).font(ui::bold(15.0)));
                });
            });
            let col = if frac > 0.9 { ui::RED } else if frac > 0.8 { ui::ORANGE } else { ui::ACCENT };
            bar(ui, ui.available_width(), frac, col);
            ui.label(caption(format!("{} free of {}", fmt_bytes(d.avail), fmt_bytes(d.total))));
        });
    }
}

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

// ------------------------------------------------------------- controls

pub fn controls(app: &mut ValleApp, ui: &mut Ui) {
    let groups = [Group::ClipboardFiles, Group::EnergyDisplay, Group::WindowsDock, Group::MouseKeyboard, Group::Sound, Group::Tools];
    for g in groups {
        let feats: Vec<&features::Feature> = features::FEATURES
            .iter()
            .filter(|f| f.group == g && f.switchable && (app.store.is_installed(f.id) || !f.support.works_here()))
            .collect();
        let extra_awake = g == Group::EnergyDisplay && app.store.is_installed("keep_awake");
        if feats.is_empty() && !extra_awake {
            continue;
        }
        let working = feats.iter().filter(|f| f.support.works_here()).count() + extra_awake as usize;
        let on = feats.iter().filter(|f| app.store.is_on(f.id)).count() + (extra_awake && app.store.keep_display_on) as usize;
        let key: &'static str = match g {
            Group::ClipboardFiles => "g_clip",
            Group::EnergyDisplay => "g_energy",
            Group::WindowsDock => "g_win",
            Group::MouseKeyboard => "g_mouse",
            Group::Sound => "g_sound",
            _ => "g_tools",
        };
        let default_open = matches!(g, Group::ClipboardFiles);
        let open = app.expanded.contains(key) ^ default_open;
        let r = ui
            .horizontal(|ui| {
                ui.label(RichText::new(if open { ph::CARET_DOWN } else { ph::CARET_RIGHT }).color(ui::MUTED));
                ui.label(RichText::new(g.title().to_uppercase()).font(ui::bold(13.0)).color(ui::MUTED).extra_letter_spacing(1.2));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let txt = if working == 0 { "later".to_string() } else { format!("{on}/{working}") };
                    ui.label(RichText::new(txt).font(ui::bold(13.0)).color(if working == 0 { ui::FAINT } else { ui::ACCENT }));
                });
            })
            .response;
        if ui.interact(r.rect, ui.id().with(key), Sense::click()).clicked() {
            if !app.expanded.remove(key) {
                app.expanded.insert(key);
            }
        }
        if !open {
            continue;
        }
        if extra_awake {
            card(ui, |ui| {
                let mut v = app.store.keep_display_on;
                ui::feature_row(ui, ph::MONITOR, "Keep display on", "While Keep awake runs, the screen stays on too.", |ui| {
                    if ui::toggle(ui, &mut v).changed() {}
                });
                if v != app.store.keep_display_on {
                    app.store.keep_display_on = v;
                    app.mark_dirty();
                }
            });
        }
        for f in feats {
            let works = f.support.works_here();
            card(ui, |ui| {
                let mut v = app.store.is_on(f.id);
                let before = v;
                ui::feature_row(ui, f.icon, f.name, f.desc, |ui| {
                    ui::toggle_enabled(ui, &mut v, works);
                });
                if !works {
                    ui.horizontal(|ui| {
                        ui.add_space(38.0);
                        ui::badge(ui, f.support.label(), ui::MUTED, ui::CONTROL);
                    });
                }
                if v != before {
                    app.store.set_on(f.id, v);
                    app.mark_dirty();
                }
                if f.id == "clip_autoclear" && app.store.is_on(f.id) {
                    ui.horizontal(|ui| {
                        ui.add_space(38.0);
                        ui.label(caption("Clear after"));
                        let mut secs = app.store.clip_auto_clear_secs.max(5);
                        egui::ComboBox::from_id_salt("autoclear")
                            .selected_text(fmt_secs(secs))
                            .show_ui(ui, |ui| {
                                for s in [15, 30, 60, 120, 300, 900] {
                                    ui.selectable_value(&mut secs, s, fmt_secs(s));
                                }
                            });
                        if secs != app.store.clip_auto_clear_secs {
                            app.store.clip_auto_clear_secs = secs;
                            app.mark_dirty();
                        }
                    });
                }
            });
        }
    }
}

pub fn fmt_secs(s: u32) -> String {
    if s < 60 { format!("{s} seconds") } else if s == 60 { "1 minute".into() } else { format!("{} minutes", s / 60) }
}
