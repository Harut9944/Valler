//! Keep awake page.

use crate::app::ValleApp;
use crate::ui::{self, caption, card, title};
use egui_phosphor::regular as ph;
use egui::Ui;
use super::*;

pub(crate) fn keep_awake(app: &mut ValleApp, ui: &mut Ui) {
    page_title(ui, "Keep awake", "Stop the computer from sleeping for a while.");
    card(ui, |ui| {
        setting_row(ui, "Keep the display on", "When off, the screen may turn off while the computer keeps working.", |ui| {
            if ui::toggle(ui, &mut app.store.keep_display_on).changed() {
                app.store.dirty_flag = true;
            }
        });
        ui::separator(ui);
        setting_row(ui, "Default duration", "", |ui| {
            let cur = crate::services::keepawake::DURATIONS.iter().find(|d| d.0 == app.store.keep_awake_minutes).map(|d| d.1).unwrap_or("1 hour");
            egui::ComboBox::from_id_salt("ka-dur").selected_text(cur).show_ui(ui, |ui| {
                for (m, l) in crate::services::keepawake::DURATIONS {
                    if ui.selectable_label(app.store.keep_awake_minutes == *m, *l).clicked() {
                        app.store.keep_awake_minutes = *m;
                        app.store.dirty_flag = true;
                    }
                }
            });
        });
    });
    automation_card(app, ui);
    app.mark_dirty_if_changed();
}

/// Keep awake automatically while chosen apps run or while plugged in.
fn automation_card(app: &mut ValleApp, ui: &mut Ui) {
    let running: Vec<String> = {
        let mut v: Vec<String> = app.snap.procs.iter().map(|p| p.name.clone()).collect();
        v.sort_by_key(|n| n.to_lowercase());
        v.dedup();
        v
    };
    let rules = &mut app.store.keep_awake_auto;
    let mut changed = false;
    card(ui, |ui| {
        ui.label(title(format!("{}  Automatically", ph::MAGIC_WAND)));
        setting_row(ui, "Keep awake automatically", "Starts and ends keep awake on its own. A session you start yourself is never ended by these rules.", |ui| {
            changed |= ui::toggle(ui, &mut rules.enabled).changed();
        });
        ui.add_enabled_ui(rules.enabled, |ui| {
            ui::separator(ui);
            setting_row(ui, "While plugged in", "Desktops without a battery always count as plugged in.", |ui| {
                changed |= ui::toggle(ui, &mut rules.power_enabled).changed();
            });
            ui::separator(ui);
            setting_row(ui, "While these apps are running", "For calls, presentations, downloads or renders.", |ui| {
                changed |= ui::toggle(ui, &mut rules.apps_enabled).changed();
            });
            if rules.apps_enabled {
                let mut remove = None;
                ui.horizontal_wrapped(|ui| {
                    for (i, a) in rules.apps.iter().enumerate() {
                        if ui.add(egui::Button::new(format!("{a}  {}", ph::X)).fill(ui::CONTROL)).on_hover_text("Remove").clicked() {
                            remove = Some(i);
                        }
                    }
                    if rules.apps.is_empty() {
                        ui.label(caption("No apps chosen yet."));
                    }
                });
                if let Some(i) = remove {
                    rules.apps.remove(i);
                    changed = true;
                }
                egui::ComboBox::from_id_salt("ka-add-app").selected_text(format!("{}  Add a running app", ph::PLUS)).width(260.0).height(320.0).show_ui(ui, |ui| {
                    for name in &running {
                        if !rules.apps.iter().any(|a| a.eq_ignore_ascii_case(name)) && ui.selectable_label(false, name).clicked() {
                            rules.apps.push(name.clone());
                            changed = true;
                        }
                    }
                });
            }
            ui::separator(ui);
            setting_row(ui, "Only when every condition holds", "Off: any one condition is enough. On: e.g. the app must run and the computer must be plugged in.", |ui| {
                changed |= ui::toggle(ui, &mut rules.require_all).changed();
            });
        });
    });
    if changed {
        app.store.dirty_flag = true;
    }
}
