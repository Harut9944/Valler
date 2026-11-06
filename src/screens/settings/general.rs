//! General page: login item, tray icon, panel tabs and settings backup.

use crate::app::ValleApp;
use crate::core::features::Tab;
use crate::core::store::Store;
use crate::ui::{self, caption, card, title};
use egui::{RichText, Ui};
use egui_phosphor::regular as ph;
use super::*;

// ------------------------------------------------------------- other pages

pub(crate) fn general(app: &mut ValleApp, ui: &mut Ui) {
    page_title(ui, "General", "How the panel looks and behaves.");
    card(ui, |ui| {
        if app.launch_at_login.is_none() {
            app.launch_at_login = Some(!app.demo && crate::services::autostart::is_enabled());
        }
        let mut on = app.launch_at_login.unwrap_or(false);
        setting_row(ui, "Open Valler when you log in", "Starts Valler automatically after you sign in to your computer.", |ui| {
            ui::toggle(ui, &mut on);
        });
        if Some(on) != app.launch_at_login && !app.demo {
            match crate::services::autostart::set(on) {
                Ok(_) => {
                    app.launch_at_login = Some(on);
                    app.toast(if on { "Valler will open at login" } else { "Valler won't open at login" });
                }
                Err(e) => app.toast(format!("Couldn't change this: {e}")),
            }
        }
    });
    tray_card(app, ui);
    card(ui, |ui| {
        ui.label(title("Panel tabs"));
        ui.label(caption("Hide tabs you don't need. A tab also disappears when all its features are uninstalled."));
        for t in Tab::ALL {
            let installed = t.features().iter().any(|f| app.store.is_installed(f));
            let mut shown = !app.store.hidden_tabs.contains(t.id());
            ui.horizontal(|ui| {
                ui.label(RichText::new(t.icon()).color(ui::ACCENT).size(17.0));
                ui.label(t.title());
                if !installed {
                    ui::badge(ui, "Not installed", ui::FAINT, ui::CONTROL);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui::toggle_enabled(ui, &mut shown, installed).changed() {
                        if shown {
                            app.store.hidden_tabs.remove(t.id());
                        } else {
                            app.store.hidden_tabs.insert(t.id().to_string());
                        }
                        app.store.dirty_flag = true;
                    }
                });
            });
        }
    });
    card(ui, |ui| {
        setting_row(ui, "Temperatures in Fahrenheit", "Show °F instead of °C.", |ui| {
            if ui::toggle(ui, &mut app.store.temperature_fahrenheit).changed() {
                app.store.dirty_flag = true;
            }
        });
    });
    card(ui, |ui| {
        ui.label(title("Your settings"));
        ui.label(caption(format!("Saved on this computer in {}", crate::core::store::config_dir().display())));
        ui.horizontal(|ui| {
            if ui::soft_button(ui, "Export settings to Documents").clicked() {
                let dir = dirs::document_dir().or_else(dirs::home_dir).unwrap_or_default();
                let path = dir.join("Valler-settings.json");
                let res = serde_json::to_string_pretty(&app.store).map_err(|e| e.to_string()).and_then(|s| std::fs::write(&path, s).map_err(|e| e.to_string()));
                app.toast(match res {
                    Ok(_) => format!("Exported to {}", path.display()),
                    Err(e) => format!("Export failed: {e}"),
                });
            }
            if ui::soft_button(ui, "Import from Documents").clicked() {
                let dir = dirs::document_dir().or_else(dirs::home_dir).unwrap_or_default();
                let path = dir.join("Valler-settings.json");
                match std::fs::read_to_string(&path).ok().and_then(|s| serde_json::from_str::<Store>(&s).ok()) {
                    Some(s) => {
                        app.store = s;
                        app.store.dirty_flag = true;
                        app.toast("Settings imported");
                    }
                    None => app.toast(format!("No settings file at {}", path.display())),
                }
            }
            if ui::soft_button(ui, "Reset everything").clicked() {
                app.store = Store::default();
                app.store.dirty_flag = true;
                app.toast("Settings reset");
            }
        });
    });
    app.mark_dirty_if_changed();
}

/// Tray / menu bar icon options.
pub(crate) fn tray_card(app: &mut ValleApp, ui: &mut Ui) {
    let place = if cfg!(target_os = "macos") { "menu bar" } else if cfg!(windows) { "taskbar notification area" } else { "system tray" };
    card(ui, |ui| {
        ui.label(title(format!("{}  Tray icon", ph::APP_WINDOW)));
        let mut on = app.store.tray_enabled;
        setting_row(ui, &format!("Show Valler in the {place}"), "Click the icon to show or hide Valler; right-click for quick actions.", |ui| {
            ui::toggle(ui, &mut on);
        });
        if on != app.store.tray_enabled {
            app.store.tray_enabled = on;
            app.store.dirty_flag = true;
            if app.demo {
            } else if on {
                app.tray = crate::services::tray::Tray::new(ui.ctx());
                if app.tray.is_none() {
                    app.toast("The tray isn't available on this desktop");
                }
            } else {
                app.tray = None;
            }
        }
        if cfg!(target_os = "linux") && !app.demo && app.store.tray_enabled && app.tray.is_none() {
            ui.label(RichText::new("No tray was found. On GNOME, turn on the AppIndicator extension (often called “Ubuntu AppIndicators” or “AppIndicator and KStatusNotifierItem Support”).").size(12.0).color(ui::ORANGE));
        }
        ui::separator(ui);
        let mut close = app.store.close_to_tray;
        setting_row(ui, "Keep running when the window is closed", "Closing the window tucks Valler into the tray, so alerts, keep awake and the clipboard keep working. Use Quit to stop it.", |ui| {
            ui::toggle_enabled(ui, &mut close, app.store.tray_enabled);
        });
        if close != app.store.close_to_tray {
            app.store.close_to_tray = close;
            app.store.dirty_flag = true;
        }
        if app.store.is_installed("tray_readouts") {
            ui::separator(ui);
            setting_row(ui, "Show on the icon", "A live gauge inside the icon and the value in its tooltip (and beside it on macOS).", |ui| {
                egui::ComboBox::from_id_salt("tray-readout").selected_text(app.store.tray_readout.label()).show_ui(ui, |ui| {
                    for r in crate::services::tray::Readout::ALL {
                        if ui.selectable_label(app.store.tray_readout == r, r.label()).clicked() {
                            app.store.tray_readout = r;
                            app.store.dirty_flag = true;
                        }
                    }
                });
            });
        }
    });
}
