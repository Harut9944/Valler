//! Volume mixer tab: apps, output device and microphone.

use crate::app::ValleApp;
use crate::services::mixer;
use crate::ui::{self, caption, card, title};
use egui::{Color32, CornerRadius, RichText, Sense, Ui, vec2};
use egui_phosphor::regular as ph;
use std::time::Instant;
use super::*;

// ------------------------------------------------------------- mixer

pub(crate) fn app_badge(ui: &mut Ui, name: &str) {
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
    let max = if app.store.mixer_boost && state.caps.boost { 2.0 } else { 1.0 };
    if let Some(m) = &state.message {
        card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(ui::icon(ph::INFO, 18.0, ui::ACCENT));
                ui.add(egui::Label::new(caption(m)).wrap());
            });
        });
    }
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
                                if !state.caps.per_app_output {
                                    return;
                                }
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
                            if !state.caps.per_app_volume {
                                ui.label(RichText::new(format!("{} Playing now", ph::WAVEFORM)).size(12.5).color(ui::GREEN));
                                return;
                            }
                            let mut v = s.volume;
                            let boosted = v > 1.0;
                            let col = if boosted { ui::ORANGE } else { ui::ACCENT };
                            ui.visuals_mut().selection.bg_fill = col;
                            ui.spacing_mut().slider_width = (ui.available_width() - 120.0).max(60.0);
                            let r = ui.add(egui::Slider::new(&mut v, 0.0..=max).show_value(false).trailing_fill(true));
                            if r.changed() {
                                app.mixer_hold = Some(Instant::now());
                                if let Ok(mut m) = app.mixer.lock()
                                    && let Some(st) = m.streams.iter_mut().find(|x| x.id == s.id) {
                                        st.volume = v;
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

pub(crate) fn device_controls(app: &mut ValleApp, ui: &mut Ui, state: &mixer::MixerState, output: bool) {
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
    if !state.caps.switch_default {
        ui.label(caption("Choosing another device opens the system's Sound settings."));
    }
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
