//! The Settings window: a sidebar of pages, with the Features hub that
//! installs and uninstalls whole features and shows platform support.

use crate::app::{SettingsPage, ValleApp};
use crate::features::{self, FEATURES, Group, PRESETS, Support, Tab};
use crate::store::Store;
use crate::tabs::fmt_secs;
use crate::ui::{self, caption, card, title};
use egui::{Color32, CornerRadius, Margin, RichText, Sense, Stroke, Ui, vec2};
use egui_phosphor::regular as ph;

const PAGES: &[(&str, &[(SettingsPage, &str, &str)])] = &[
    ("Essentials", &[
        (SettingsPage::General, ph::GEAR, "General"),
        (SettingsPage::Features, ph::SQUARES_FOUR, "Features"),
        (SettingsPage::Monitor, ph::CHART_LINE, "Monitor"),
        (SettingsPage::KeepAwake, ph::MOON_STARS, "Keep awake"),
    ]),
    ("Files", &[
        (SettingsPage::Clipboard, ph::CLIPBOARD_TEXT, "Clipboard"),
        (SettingsPage::Snippets, ph::TEXT_AA, "Snippets"),
        (SettingsPage::UrlClean, ph::LINK, "Clean URL"),
    ]),
    ("Valler", &[(SettingsPage::About, ph::INFO, "About")]),
];

pub fn show(app: &mut ValleApp, ui: &mut Ui, _ctx: &egui::Context) {
    // The whole window, measured before any panel takes its share.
    let window = ui.max_rect();
    let r = ui::WINDOW_RADIUS;
    let side_fill = Color32::from_rgb(0x22, 0x22, 0x25);
    egui::Panel::left("settings-sidebar").show_separator_line(false)
        .exact_size(230.0)
        .resizable(false)
        .frame(
            egui::Frame::new()
                .fill(side_fill)
                .corner_radius(CornerRadius { nw: r, sw: r, ne: 0, se: 0 })
                .inner_margin(Margin { left: 12, right: 12, top: 14, bottom: 14 }),
        )
        .show(ui, |ui| {
            // The top strip of the sidebar moves the window too.
            let strip = egui::Rect::from_min_size(ui.max_rect().min - vec2(12.0, 14.0), vec2(230.0, 12.0));
            ui::drag_area(ui, strip, "settings-drag-side", true);
            sidebar(app, ui)
        });
    // Title bar: drag area, title and the window buttons (no system frame).
    let mut close = false;
    egui::Panel::top("settings-titlebar")
        .show_separator_line(false)
        .exact_size(40.0)
        .frame(egui::Frame::new().inner_margin(Margin { left: 28, right: 12, top: 8, bottom: 0 }))
        .show(ui, |ui| {
            // Span the title bar from the sidebar to the real right edge of the window,
            // so the window buttons sit inside the rounded border.
            let top = ui.max_rect();
            let bar = egui::Rect::from_min_max(egui::pos2(top.left() - 28.0, window.top()), egui::pos2(window.right(), top.bottom()));
            ui::drag_area(ui, bar, "settings-drag", true);
            ui.label(RichText::new("Valler Settings").font(ui::bold(13.5)).color(ui::MUTED));
            close = ui::window_buttons(ui, bar.shrink2(vec2(6.0, 0.0)).translate(vec2(-2.0, 2.0)), "settings");
        });
    if close {
        app.settings_open = false;
    }
    egui::CentralPanel::default()
        .frame(egui::Frame::new().inner_margin(Margin { left: 28, right: 28, top: 4, bottom: 18 }))
        .show(ui, |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                let w = ui.available_width().min(860.0);
                ui.set_max_width(w);
                ui.spacing_mut().item_spacing.y = 10.0;
                match app.settings_page {
                    SettingsPage::General => general(app, ui),
                    SettingsPage::Features => hub(app, ui),
                    SettingsPage::Monitor => monitor(app, ui),
                    SettingsPage::KeepAwake => keep_awake(app, ui),
                    SettingsPage::Clipboard => clipboard(app, ui),
                    SettingsPage::Snippets => snippets(app, ui),
                    SettingsPage::UrlClean => url_clean(app, ui),
                    SettingsPage::About => about(app, ui),
                }
                ui.add_space(20.0);
            });
        });
}

fn sidebar(app: &mut ValleApp, ui: &mut Ui) {
    ui.add(
        egui::TextEdit::singleline(&mut app.settings_search)
            .hint_text(format!("{}  Search settings", ph::MAGNIFYING_GLASS))
            .desired_width(f32::INFINITY)
            .margin(vec2(10.0, 7.0)),
    );
    ui.add_space(8.0);
    let q = app.settings_search.to_lowercase();
    for (section, pages) in PAGES {
        let visible: Vec<_> = pages.iter().filter(|(_, _, n)| q.is_empty() || n.to_lowercase().contains(&q)).collect();
        if visible.is_empty() {
            continue;
        }
        ui.add_space(6.0);
        ui.label(RichText::new(*section).font(ui::bold(12.5)).color(ui::FAINT));
        for (page, glyph, name) in visible {
            let sel = app.settings_page == *page;
            let w = ui.available_width();
            let (r, resp) = ui.allocate_exact_size(vec2(w, 34.0), Sense::click());
            let p = ui.painter();
            if sel {
                p.rect_filled(r, CornerRadius::same(8), ui::ACCENT);
            } else if resp.hovered() {
                p.rect_filled(r, CornerRadius::same(8), ui::CARD_HOVER);
            }
            let ic = if sel { Color32::WHITE } else { ui::ACCENT };
            p.text(r.left_center() + vec2(14.0, 0.0), egui::Align2::LEFT_CENTER, *glyph, egui::FontId::proportional(17.0), ic);
            p.text(r.left_center() + vec2(42.0, 0.0), egui::Align2::LEFT_CENTER, *name, egui::FontId::proportional(15.0), if sel { Color32::WHITE } else { ui::TEXT });
            if resp.clicked() {
                app.settings_page = *page;
            }
        }
    }
    // Features matching a search jump straight to the hub.
    if !q.is_empty() {
        let hits: Vec<_> = FEATURES.iter().filter(|f| f.name.to_lowercase().contains(&q) || f.desc.to_lowercase().contains(&q)).take(6).collect();
        if !hits.is_empty() {
            ui.add_space(8.0);
            ui.label(RichText::new("Features").font(ui::bold(12.5)).color(ui::FAINT));
            for f in hits {
                if ui.add(egui::Button::new(RichText::new(format!("{}  {}", f.icon, f.name))).frame(false)).clicked() {
                    app.settings_page = SettingsPage::Features;
                }
            }
        }
    }
}

fn page_title(ui: &mut Ui, t: &str, sub: &str) {
    ui.label(RichText::new(t).font(ui::bold(24.0)));
    if !sub.is_empty() {
        ui.label(caption(sub));
    }
    ui.add_space(4.0);
}

fn setting_row(ui: &mut Ui, name: &str, desc: &str, right: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_width((ui.available_width() - 200.0).max(200.0));
            ui.spacing_mut().item_spacing.y = 1.0;
            ui.label(RichText::new(name).size(14.5));
            if !desc.is_empty() {
                ui.add(egui::Label::new(caption(desc)).wrap());
            }
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), right);
    });
}

// ------------------------------------------------------------- features hub

fn segmented(ui: &mut Ui, left: &str, right: &str, right_selected: &mut bool) {
    egui::Frame::new()
        .fill(ui::CONTROL)
        .corner_radius(CornerRadius::same(9))
        .inner_margin(Margin::same(3))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                for (i, t) in [left, right].iter().enumerate() {
                    let sel = (i == 1) == *right_selected;
                    let b = egui::Button::new(RichText::new(*t).size(15.0).color(if sel { Color32::WHITE } else { ui::TEXT }))
                        .fill(if sel { ui::ACCENT } else { Color32::TRANSPARENT })
                        .corner_radius(CornerRadius::same(7))
                        .min_size(vec2(130.0, 32.0));
                    if ui.add(b).clicked() {
                        *right_selected = i == 1;
                    }
                }
            });
        });
}

fn hub(app: &mut ValleApp, ui: &mut Ui) {
    card(ui, |ui| {
        segmented(ui, "Features", "Platform support", &mut app.hub_tab_support);
        ui::separator(ui);
        if app.hub_tab_support {
            ui.label(caption("What works on each system in this version. Features marked Coming later are planned and appear in the panel greyed out."));
        } else {
            ui.label(caption("Install only what you use. Whatever you uninstall disappears from the whole app and stops running."));
        }
        ui::separator(ui);
        let installable: Vec<_> = FEATURES.iter().filter(|f| f.support.works_here()).collect();
        let n = installable.iter().filter(|f| app.store.is_installed(f.id)).count();
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("{n} of {} features installed", installable.len())).color(ui::MUTED));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add_enabled(n > 0, egui::Button::new("Uninstall all").fill(ui::CONTROL)).clicked() {
                    app.store.installed.clear();
                    app.mark_dirty();
                }
                if ui.add_enabled(n < installable.len(), egui::Button::new("Install all").fill(ui::CONTROL)).clicked() {
                    app.store.installed = features::default_installed();
                    app.mark_dirty();
                }
            });
        });
    });

    if app.hub_tab_support {
        support_table(ui);
        return;
    }

    ui.add_space(6.0);
    ui.label(RichText::new("Start with a bundle").font(ui::bold(17.0)));
    card(ui, |ui| {
        let w = (ui.available_width() - 2.0 * 10.0) / 3.0;
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            for p in PRESETS {
                egui::Frame::new()
                    .fill(Color32::from_rgb(0x31, 0x31, 0x35))
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(Margin::same(12))
                    .show(ui, |ui| {
                        ui.set_width(w - 24.0);
                        ui.set_min_height(108.0);
                        ui.vertical(|ui| {
                            ui.label(RichText::new(format!("{}  {}", p.icon, p.name)).font(ui::bold(15.5)).color(ui::TEXT));
                            ui.add(egui::Label::new(caption(p.desc)).wrap());
                            ui.add_space(4.0);
                            if ui.add(egui::Button::new("Apply").fill(ui::CONTROL)).clicked() {
                                app.store.installed = p.features.iter().filter(|id| features::get(id).is_some_and(|f| f.support.works_here())).map(|s| s.to_string()).collect();
                                app.mark_dirty();
                                app.toast(format!("Applied {}", p.name));
                            }
                        });
                    });
            }
        });
    });
    ui.label(caption("One click sets the app up for how you use your computer. Everything else stays one click away."));

    for g in Group::ALL {
        ui.add_space(8.0);
        ui.label(RichText::new(g.title()).font(ui::bold(17.0)));
        card(ui, |ui| {
            let list: Vec<_> = FEATURES.iter().filter(|f| f.group == g).collect();
            for (i, f) in list.iter().enumerate() {
                if i > 0 {
                    ui::separator(ui);
                }
                hub_row(&mut app.store, ui, f);
            }
        });
    }
    if let Some(t) = app.toast.as_ref() {
        let _ = t;
    }
    // Keep panel tab selection valid after uninstalling.
    if !app.tab_visible(app.tab) {
        if let Some(t) = app.visible_tabs().first() {
            app.tab = *t;
            app.sub = None;
        }
    }
    app.mark_dirty_if_changed();
}

fn hub_row(store: &mut Store, ui: &mut Ui, f: &features::Feature) {
    let works = f.support.works_here();
    ui.horizontal(|ui| {
        let (r, _) = ui.allocate_exact_size(vec2(40.0, 40.0), Sense::hover());
        ui.painter().rect_filled(r, CornerRadius::same(9), Color32::BLACK);
        ui.painter().text(r.center(), egui::Align2::CENTER_CENTER, f.icon, egui::FontId::proportional(21.0), if works { Color32::WHITE } else { ui::FAINT });
        ui.vertical(|ui| {
            ui.set_width((ui.available_width() - 120.0).max(200.0));
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.horizontal(|ui| {
                ui.label(RichText::new(f.name).size(16.0).color(if works { ui::TEXT } else { ui::MUTED }));
                ui::badge(ui, f.energy, ui::MUTED, ui::CONTROL);
                let (fg, bg) = match f.support {
                    Support::Later => (ui::FAINT, ui::CONTROL),
                    Support::Partial(_) => (ui::YELLOW, ui::YELLOW.gamma_multiply(0.15)),
                    _ if works => (ui::GREEN, ui::GREEN.gamma_multiply(0.15)),
                    _ => (ui::ORANGE, ui::ORANGE.gamma_multiply(0.15)),
                };
                let r = ui.scope(|ui| ui::badge(ui, f.support.label(), fg, bg)).response;
                if let Support::Partial(note) = f.support {
                    r.on_hover_text(note);
                }
            });
            ui.add(egui::Label::new(caption(f.desc)).wrap());
            if let Support::Partial(note) = f.support {
                ui.add(egui::Label::new(RichText::new(note).size(11.5).color(ui::FAINT)).wrap());
            }
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if !works {
                ui.add_enabled(false, egui::Button::new("Coming later").fill(ui::CONTROL));
            } else if store.is_installed(f.id) {
                if ui.add(egui::Button::new("Uninstall").fill(ui::CONTROL).stroke(Stroke::new(1.0, ui::BORDER))).clicked() {
                    store.installed.remove(f.id);
                    store.dirty_flag = true;
                }
            } else if ui.add(egui::Button::new(RichText::new("Install").color(Color32::WHITE)).fill(ui::ACCENT)).clicked() {
                store.installed.insert(f.id.to_string());
                store.dirty_flag = true;
            }
        });
    });
}

fn support_table(ui: &mut Ui) {
    let mark = |s: Support, os: &str| -> (&'static str, Color32) {
        match s {
            Support::Everywhere => (ph::CHECK_CIRCLE, ui::GREEN),
            Support::Partial(_) => (ph::CHECK_CIRCLE, ui::YELLOW),
            Support::LinuxOnly if os == "linux" => (ph::CHECK_CIRCLE, ui::GREEN),
            Support::LinuxOnly | Support::Later => (ph::CIRCLE_DASHED, ui::FAINT),
        }
    };
    for g in Group::ALL {
        ui.add_space(6.0);
        ui.label(RichText::new(g.title()).font(ui::bold(17.0)));
        card(ui, |ui| {
            egui::Grid::new(("support", g.title())).num_columns(4).spacing(vec2(24.0, 8.0)).striped(false).show(ui, |ui| {
                ui.label(RichText::new("Feature").color(ui::MUTED));
                for os in ["Windows", "Linux (GNOME, KDE)", "macOS (Apple silicon, Intel)"] {
                    ui.label(RichText::new(os).color(ui::MUTED));
                }
                ui.end_row();
                for f in FEATURES.iter().filter(|f| f.group == g) {
                    ui.label(format!("{}  {}", f.icon, f.name));
                    for os in ["windows", "linux", "macos"] {
                        let (g, c) = mark(f.support, os);
                        let r = ui.label(RichText::new(g).color(c).size(17.0));
                        if let Support::Partial(n) = f.support {
                            r.on_hover_text(n);
                        }
                    }
                    ui.end_row();
                }
            });
        });
    }
    ui.label(caption(format!(
        "{} Works   {} Works with limits (hover for details)   {} Coming later",
        ph::CHECK_CIRCLE,
        ph::CHECK_CIRCLE,
        ph::CIRCLE_DASHED
    )));
}

// ------------------------------------------------------------- other pages

fn general(app: &mut ValleApp, ui: &mut Ui) {
    page_title(ui, "General", "How the panel looks and behaves.");
    card(ui, |ui| {
        if app.launch_at_login.is_none() {
            app.launch_at_login = Some(!app.demo && crate::autostart::is_enabled());
        }
        let mut on = app.launch_at_login.unwrap_or(false);
        setting_row(ui, "Open Valler when you log in", "Starts Valler automatically after you sign in to your computer.", |ui| {
            ui::toggle(ui, &mut on);
        });
        if Some(on) != app.launch_at_login && !app.demo {
            match crate::autostart::set(on) {
                Ok(_) => {
                    app.launch_at_login = Some(on);
                    app.toast(if on { "Valler will open at login" } else { "Valler won't open at login" });
                }
                Err(e) => app.toast(format!("Couldn't change this: {e}")),
            }
        }
    });
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
        ui.label(caption(format!("Saved on this computer in {}", crate::store::config_dir().display())));
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

fn monitor(app: &mut ValleApp, ui: &mut Ui) {
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
fn alert_row(ui: &mut Ui, on: &mut bool, name: &str, desc: &str, limit: Option<(&mut u32, std::ops::RangeInclusive<u32>, &str)>) -> bool {
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

fn alerts_card(app: &mut ValleApp, ui: &mut Ui) {
    let deg = if app.store.temperature_fahrenheit { " °F" } else { " °C" };
    card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(title(format!("{}  Alerts", ph::BELL_RINGING)));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui::soft_button(ui, "Send a test notification").clicked() {
                    crate::alerts::notify("Valler alerts are working", "You'll see notifications like this when a limit is crossed.");
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

fn keep_awake(app: &mut ValleApp, ui: &mut Ui) {
    page_title(ui, "Keep awake", "Stop the computer from sleeping for a while.");
    card(ui, |ui| {
        setting_row(ui, "Keep the display on", "When off, the screen may turn off while the computer keeps working.", |ui| {
            if ui::toggle(ui, &mut app.store.keep_display_on).changed() {
                app.store.dirty_flag = true;
            }
        });
        ui::separator(ui);
        setting_row(ui, "Default duration", "", |ui| {
            let cur = crate::keepawake_ctl::DURATIONS.iter().find(|d| d.0 == app.store.keep_awake_minutes).map(|d| d.1).unwrap_or("1 hour");
            egui::ComboBox::from_id_salt("ka-dur").selected_text(cur).show_ui(ui, |ui| {
                for (m, l) in crate::keepawake_ctl::DURATIONS {
                    if ui.selectable_label(app.store.keep_awake_minutes == *m, *l).clicked() {
                        app.store.keep_awake_minutes = *m;
                        app.store.dirty_flag = true;
                    }
                }
            });
        });
    });
    app.mark_dirty_if_changed();
}

fn clipboard(app: &mut ValleApp, ui: &mut Ui) {
    page_title(ui, "Clipboard", "History stays on this computer and can be cleared anytime.");
    card(ui, |ui| {
        let mut on = app.store.is_on("clipboard");
        setting_row(ui, "Save clipboard history", "Keeps text you copy so you can reuse it from the panel.", |ui| {
            ui::toggle(ui, &mut on);
        });
        if on != app.store.is_on("clipboard") {
            app.store.set_on("clipboard", on);
            app.store.dirty_flag = true;
        }
        ui::separator(ui);
        setting_row(ui, "Items to keep", "Pinned items are always kept.", |ui| {
            let mut n = app.store.clip_max_items as u32;
            if ui.add(egui::DragValue::new(&mut n).range(10..=2000)).changed() {
                app.store.clip_max_items = n as usize;
                app.store.dirty_flag = true;
            }
        });
        ui::separator(ui);
        let mut ac = app.store.is_on("clip_autoclear");
        setting_row(ui, "Clear the clipboard automatically", "Empties the system clipboard a while after you copy. History is kept.", |ui| {
            ui::toggle(ui, &mut ac);
        });
        if ac != app.store.is_on("clip_autoclear") {
            app.store.set_on("clip_autoclear", ac);
            app.store.dirty_flag = true;
        }
        if ac {
            setting_row(ui, "Clear after", "", |ui| {
                let mut secs = app.store.clip_auto_clear_secs.max(5);
                egui::ComboBox::from_id_salt("s-autoclear").selected_text(fmt_secs(secs)).show_ui(ui, |ui| {
                    for s in [15, 30, 60, 120, 300, 900] {
                        ui.selectable_value(&mut secs, s, fmt_secs(s));
                    }
                });
                if secs != app.store.clip_auto_clear_secs {
                    app.store.clip_auto_clear_secs = secs;
                    app.store.dirty_flag = true;
                }
            });
        }
        ui::separator(ui);
        ui.horizontal(|ui| {
            ui.label(caption(format!("{} items saved", app.store.clip_history.len())));
            if ui::soft_button(ui, "Clear history").clicked() {
                app.store.clip_history.clear();
                app.store.dirty_flag = true;
            }
        });
    });
    app.mark_dirty_if_changed();
}

fn snippets(app: &mut ValleApp, ui: &mut Ui) {
    page_title(ui, "Snippets", "Saved text you can copy in one click from the panel.");
    card(ui, |ui| {
        ui.label(caption("Variables: {{date}}, {{time}}, {{datetime}}, {{clipboard}}, {{name}}. Add a format after a colon, like {{date:%A %d %B}}."));
        ui.label(caption("Typing a trigger anywhere to expand it needs a keyboard hook, which is coming in a later version."));
    });
    let mut remove = None;
    for (i, s) in app.store.snippets.iter_mut().enumerate() {
        card(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.add(egui::TextEdit::singleline(&mut s.trigger).desired_width(160.0)).changed() {
                    app.store.dirty_flag = true;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(RichText::new(ph::TRASH).color(ui::MUTED)).frame(false)).clicked() {
                        remove = Some(i);
                    }
                });
            });
            if ui.add(egui::TextEdit::multiline(&mut s.text).desired_rows(2).desired_width(f32::INFINITY)).changed() {
                app.store.dirty_flag = true;
            }
        });
    }
    if let Some(i) = remove {
        app.store.snippets.remove(i);
        app.store.dirty_flag = true;
    }
    if ui::primary_button(ui, format!("{} Add snippet", ph::PLUS)).clicked() {
        app.store.snippets.push(crate::store::Snippet { trigger: ";new".into(), text: String::new() });
        app.store.dirty_flag = true;
    }
    app.mark_dirty_if_changed();
}

fn url_clean(app: &mut ValleApp, ui: &mut Ui) {
    page_title(ui, "Clean URL", "Removes tracking parameters such as utm_source, fbclid and gclid.");
    card(ui, |ui| {
        let mut on = app.store.is_on("url_clean");
        setting_row(ui, "Clean copied links automatically", "Each link you copy is replaced by its clean version.", |ui| {
            ui::toggle(ui, &mut on);
        });
        if on != app.store.is_on("url_clean") {
            app.store.set_on("url_clean", on);
            app.store.dirty_flag = true;
        }
        ui::separator(ui);
        ui.label(RichText::new("Extra parameters to remove").size(14.5));
        ui.label(caption("Comma separated, applied on every site."));
        if ui.add(egui::TextEdit::singleline(&mut app.store.url_extra_params).hint_text("ref, source, campaign").desired_width(f32::INFINITY)).changed() {
            app.store.dirty_flag = true;
        }
    });
    card(ui, |ui| {
        ui.label(title("Built-in rules"));
        ui.label(caption("Everywhere: every utm_ parameter, fbclid, gclid, dclid, msclkid, yclid, igshid, mc_cid, mc_eid, twclid, ttclid and more."));
        ui.label(caption("Site-specific: YouTube (si, pp, feature), X and Twitter (s, t, src), Instagram (igsh), Spotify (si), Reddit, TikTok and Amazon share parameters."));
    });
    app.mark_dirty_if_changed();
}

fn about(app: &mut ValleApp, ui: &mut Ui) {
    let _ = app;
    ui.add_space(20.0);
    ui.vertical_centered(|ui| {
        let (r, _) = ui.allocate_exact_size(vec2(100.0, 80.0), Sense::hover());
        ui::paint_logo(ui.painter(), r.center(), 80.0);
        ui.label(RichText::new("Valler").font(ui::bold(28.0)));
        ui.label(caption(format!("Version {}", env!("CARGO_PKG_VERSION"))));
        ui.add_space(8.0);
        ui.label("Everyday desktop utilities in one window, for Windows, Linux and macOS.");
        ui.label(caption("No account, no analytics. The network is only used for the speed test you start yourself."));
        ui.add_space(8.0);
        ui.label(caption("Free software under the GNU GPL 3.0 or later. A cross-platform rewrite in Rust of an open-source macOS utility collection."));
    });
}

impl ValleApp {
    /// Pages edit the store directly and set `dirty_flag`; this folds it into the save timer.
    pub fn mark_dirty_if_changed(&mut self) {
        if self.store.dirty_flag {
            self.store.dirty_flag = false;
            self.mark_dirty();
        }
    }
}

#[allow(dead_code)]
fn unused(_: &mut Ui) {
    let _ = Margin::ZERO;

}
