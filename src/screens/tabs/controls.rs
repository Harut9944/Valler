//! Controls tab: on/off switches grouped like the Features hub.

use crate::app::ValleApp;
use crate::core::features::{self, Group};
use crate::ui::{self, caption, card};
use egui::{RichText, Sense, Ui};
use egui_phosphor::regular as ph;
use super::*;

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
        if ui.interact(r.rect, ui.id().with(key), Sense::click()).clicked()
            && !app.expanded.remove(key) {
                app.expanded.insert(key);
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
