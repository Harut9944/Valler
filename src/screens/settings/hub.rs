//! Features hub: install/uninstall features, bundles and platform support.

use crate::app::ValleApp;
use crate::core::features::{self, FEATURES, Group, PRESETS, Support};
use crate::core::store::Store;
use crate::ui::{self, caption, card};
use egui::{Color32, CornerRadius, Margin, RichText, Sense, Stroke, Ui, vec2};
use egui_phosphor::regular as ph;
use super::*;

pub(crate) fn hub(app: &mut ValleApp, ui: &mut Ui) {
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
    if !app.tab_visible(app.tab)
        && let Some(t) = app.visible_tabs().first() {
            app.tab = *t;
            app.sub = None;
        }
    app.mark_dirty_if_changed();
}

pub(crate) fn hub_row(store: &mut Store, ui: &mut Ui, f: &features::Feature) {
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

pub(crate) fn support_table(ui: &mut Ui) {
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
