//! Package manager / Uninstaller / App updates and Media tools screens.

use crate::app::ValleApp;
use crate::media::{self, Format};
use crate::packages::{self, Op, Package};
use crate::ui::{self, caption, card, fmt_bytes, title};
use egui::{Color32, CornerRadius, RichText, Ui};
use egui_phosphor::regular as ph;

// ------------------------------------------------------------- packages

fn tabs(ui: &mut Ui, sel: &mut u8, labels: &[String]) {
    egui::Frame::new()
        .fill(ui::CONTROL)
        .corner_radius(CornerRadius::same(9))
        .inner_margin(egui::Margin::same(3))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                let w = (ui.available_width() - 2.0 * (labels.len() as f32 - 1.0)) / labels.len() as f32;
                for (i, l) in labels.iter().enumerate() {
                    let on = *sel == i as u8;
                    let b = egui::Button::new(RichText::new(l).size(13.5).color(if on { Color32::WHITE } else { ui::TEXT }))
                        .fill(if on { ui::ACCENT } else { Color32::TRANSPARENT })
                        .corner_radius(CornerRadius::same(7))
                        .min_size(egui::vec2(w, 28.0));
                    if ui.add(b).clicked() {
                        *sel = i as u8;
                    }
                }
            });
        });
}

fn pkg_row(ui: &mut Ui, p: &Package, action: &mut Option<(Package, Op)>, ops: &[(Op, &str, Color32)]) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_width(ui.available_width() - 90.0);
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.add(egui::Label::new(RichText::new(&p.name).size(14.0)).truncate());
            let mut detail = format!("{} · {}", p.backend.label(), p.id);
            if !p.version.is_empty() && p.new_version.is_empty() {
                detail.push_str(&format!(" · {}", p.version));
            }
            if !p.new_version.is_empty() {
                detail.push_str(&format!(" · {} → {}", if p.version.is_empty() { "?" } else { &p.version }, p.new_version));
            }
            ui.add(egui::Label::new(RichText::new(detail).size(11.5).color(ui::FAINT)).truncate());
            if !p.desc.is_empty() {
                ui.add(egui::Label::new(RichText::new(&p.desc).size(11.5).color(ui::MUTED)).truncate());
            }
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            for (op, label, col) in ops {
                if ui.add(egui::Button::new(RichText::new(*label).size(12.5).color(*col)).fill(ui::CONTROL)).clicked() {
                    *action = Some((p.clone(), *op));
                }
            }
        });
    });
}

/// `forced_tab`: the Uninstaller card opens straight on the installed list.
pub fn packages_view(app: &mut ValleApp, ui: &mut Ui, ctx: &egui::Context, forced_tab: Option<u8>) {
    if let Some(t) = forced_tab {
        if !app.pkg_tab_forced {
            app.pkg_tab = t;
            app.pkg_tab_forced = true;
        }
    }
    let stale = ctx.data_mut(|d| d.remove_temp::<bool>(egui::Id::new("packages-stale")).unwrap_or(false));
    let st = app.pkg.lock().map(|s| s.clone()).unwrap_or_default();
    if app.demo {
        if st.installed.is_none() {
            if let Ok(mut s) = app.pkg.lock() {
                *s = packages::demo();
            }
        }
    } else if (st.installed.is_none() && st.busy.is_none()) || (stale && st.busy.is_none()) {
        packages::refresh(app.pkg.clone(), ctx.clone());
    }
    let st = app.pkg.lock().map(|s| s.clone()).unwrap_or_default();

    if st.installed.is_some() && st.backends.is_empty() {
        card(ui, |ui| {
            ui::empty_note(
                ui,
                ph::PACKAGE,
                if cfg!(windows) {
                    "winget wasn't found. It comes with the App Installer from the Microsoft Store."
                } else if cfg!(target_os = "macos") {
                    "Homebrew wasn't found. Install it from brew.sh to manage apps here."
                } else {
                    "Neither Flatpak nor Homebrew was found. Install Flatpak from your distribution to manage apps here."
                },
            );
        });
        return;
    }
    let n_up = st.updates.as_ref().map(|u| u.len()).unwrap_or(0);
    let labels = vec![
        format!("Installed ({})", st.installed.as_ref().map(|v| v.len()).unwrap_or(0)),
        if n_up > 0 { format!("Updates ({n_up})") } else { "Updates".into() },
        "Get apps".into(),
    ];
    tabs(ui, &mut app.pkg_tab, &labels);
    if !st.backends.is_empty() {
        ui.label(caption(format!("Using {}", st.backends.iter().map(|b| b.label()).collect::<Vec<_>>().join(", "))));
    }
    if let Some(b) = &st.busy {
        card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(b);
            });
        });
    }
    if let Some(m) = &st.message {
        ui.label(RichText::new(m).size(12.5).color(if m.starts_with("Couldn't") { ui::ORANGE } else { ui::GREEN }));
    }
    let busy = st.busy.is_some();
    let mut action: Option<(Package, Op)> = None;
    match app.pkg_tab {
        0 => {
            ui.add(egui::TextEdit::singleline(&mut app.pkg_filter).hint_text(format!("{}  Filter installed apps", ph::MAGNIFYING_GLASS)).desired_width(f32::INFINITY));
            let q = app.pkg_filter.to_lowercase();
            card(ui, |ui| {
                let list: Vec<&Package> = st.installed.iter().flatten().filter(|p| q.is_empty() || p.name.to_lowercase().contains(&q) || p.id.to_lowercase().contains(&q)).collect();
                if list.is_empty() {
                    ui::empty_note(ui, ph::PACKAGE, if st.installed.is_none() { "Reading…" } else { "No apps found." });
                }
                ui.add_enabled_ui(!busy, |ui| {
                    for p in list.into_iter().take(150) {
                        pkg_row(ui, p, &mut action, &[(Op::Remove, "Uninstall", ui::RED)]);
                    }
                });
            });
        }
        1 => {
            card(ui, |ui| {
                let ups = st.updates.clone().unwrap_or_default();
                ui.horizontal(|ui| {
                    ui.label(title(if ups.is_empty() { "Everything is up to date" } else { "Updates available" }));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add_enabled(!busy && !ups.is_empty() && !app.demo, egui::Button::new(RichText::new("Update all").color(Color32::WHITE)).fill(ui::ACCENT)).clicked() {
                            packages::update_all(app.pkg.clone(), ctx.clone());
                        }
                        if ui.add_enabled(!busy && !app.demo, egui::Button::new(RichText::new(ph::ARROWS_CLOCKWISE)).fill(ui::CONTROL)).on_hover_text("Check again").clicked() {
                            packages::refresh(app.pkg.clone(), ctx.clone());
                        }
                    });
                });
                ui.add_enabled_ui(!busy, |ui| {
                    for p in &ups {
                        ui::separator(ui);
                        pkg_row(ui, p, &mut action, &[(Op::Update, "Update", ui::ACCENT)]);
                    }
                });
            });
        }
        _ => {
            let r = ui.add(egui::TextEdit::singleline(&mut app.pkg_query).hint_text(format!("{}  Search apps to install", ph::MAGNIFYING_GLASS)).desired_width(f32::INFINITY).margin(egui::vec2(10.0, 7.0)));
            if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !app.pkg_query.trim().is_empty() && !app.demo {
                packages::search(app.pkg.clone(), ctx.clone(), app.pkg_query.trim().to_string());
            }
            if let Some(res) = &st.results {
                card(ui, |ui| {
                    if res.is_empty() {
                        ui::empty_note(ui, ph::MAGNIFYING_GLASS, "No matches.");
                    }
                    ui.add_enabled_ui(!busy, |ui| {
                        for p in res {
                            let installed = st.installed.iter().flatten().any(|i| i.id == p.id);
                            if installed {
                                pkg_row(ui, p, &mut action, &[(Op::Remove, "Uninstall", ui::RED)]);
                            } else {
                                pkg_row(ui, p, &mut action, &[(Op::Install, "Install", ui::ACCENT)]);
                            }
                        }
                    });
                });
            } else {
                ui.label(caption("Type a name and press Enter. Apps come from the stores your system already uses."));
            }
        }
    }
    if let Some(a) = action {
        if a.1 == Op::Remove {
            app.pkg_confirm = Some(a);
        } else if !app.demo {
            packages::act(app.pkg.clone(), ctx.clone(), a.0, a.1);
        }
    }
    if let Some((p, op)) = app.pkg_confirm.clone() {
        card(ui, |ui| {
            ui.label(title(format!("Uninstall {}?", p.name)));
            ui.label(caption("The app is removed. Its settings in your home folder are kept."));
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new(RichText::new("Uninstall").color(Color32::WHITE)).fill(ui::RED)).clicked() {
                    if !app.demo {
                        packages::act(app.pkg.clone(), ctx.clone(), p.clone(), op);
                    }
                    app.pkg_confirm = None;
                }
                if ui::soft_button(ui, "Cancel").clicked() {
                    app.pkg_confirm = None;
                }
            });
        });
    }
}

// ------------------------------------------------------------- media tools

pub fn media_view(app: &mut ValleApp, ui: &mut Ui, ctx: &egui::Context) {
    let prog = app.media.lock().map(|p| p.clone()).unwrap_or_default();
    card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(ui::icon(ph::IMAGES, 30.0, ui::ACCENT));
            ui.vertical(|ui| {
                ui.label(title("Convert and compress images"));
                ui.add(egui::Label::new(caption("Change the format, shrink large photos and reduce file size. Everything happens on this computer; originals are kept.")).wrap());
            });
        });
        ui::separator(ui);
        ui.horizontal(|ui| {
            if ui::soft_button(ui, format!("{} Choose images…", ph::FOLDER_OPEN)).clicked() && !app.demo {
                let picked = media::pick_files();
                if !picked.is_empty() {
                    app.media_files = picked;
                }
            }
            if !app.media_files.is_empty() && ui.add(egui::Button::new(RichText::new("Clear").size(12.5)).frame(false)).clicked() {
                app.media_files.clear();
            }
        });
        // Files dropped on the window are added too.
        let dropped: Vec<std::path::PathBuf> = ctx.input(|i| i.raw.dropped_files.iter().map(|f| f.path().to_path_buf()).filter(|p| !p.as_os_str().is_empty()).collect());
        for p in dropped {
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            if media::EXTENSIONS.contains(&ext.as_str()) && !app.media_files.contains(&p) {
                app.media_files.push(p);
            }
        }
        if app.media_files.is_empty() {
            ui.label(caption("Or drop images onto this window."));
        } else {
            let total: u64 = app.media_files.iter().filter_map(|f| std::fs::metadata(f).ok()).map(|m| m.len()).sum();
            ui.label(RichText::new(format!("{} image{} · {}", app.media_files.len(), if app.media_files.len() == 1 { "" } else { "s" }, fmt_bytes(total))).font(ui::bold(14.0)));
            for f in app.media_files.iter().take(6) {
                ui.add(egui::Label::new(RichText::new(f.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()).size(12.0).color(ui::MUTED)).truncate());
            }
            if app.media_files.len() > 6 {
                ui.label(caption(format!("and {} more", app.media_files.len() - 6)));
            }
        }
        ui::separator(ui);
        ui.horizontal(|ui| {
            ui.label("Format");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                egui::ComboBox::from_id_salt("media-format").selected_text(app.media_format.label()).show_ui(ui, |ui| {
                    for f in Format::ALL {
                        ui.selectable_value(&mut app.media_format, f, f.label());
                    }
                });
            });
        });
        ui.horizontal(|ui| {
            ui.label("Largest side");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let label = |v: u32| if v == 0 { "Keep size".to_string() } else { format!("{v} px") };
                egui::ComboBox::from_id_salt("media-size").selected_text(label(app.media_max)).show_ui(ui, |ui| {
                    for v in [0u32, 3840, 2560, 1920, 1280, 800] {
                        ui.selectable_value(&mut app.media_max, v, label(v));
                    }
                });
            });
        });
        if app.media_format == Format::Jpeg {
            ui.horizontal(|ui| {
                ui.label("Quality");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(egui::Slider::new(&mut app.media_quality, 30..=100).suffix(" %"));
                });
            });
        }
        ui.add_space(4.0);
        let w = ui.available_width();
        let can = !app.media_files.is_empty() && !prog.running;
        let b = egui::Button::new(RichText::new(format!("{}  Convert", ph::ARROWS_CLOCKWISE)).font(ui::bold(14.5)).color(Color32::WHITE))
            .fill(ui::ACCENT)
            .corner_radius(CornerRadius::same(10))
            .min_size(egui::vec2(w, 36.0));
        if ui.add_enabled(can, b).clicked() {
            media::start(
                app.media.clone(),
                ctx.clone(),
                media::Job { files: app.media_files.clone(), format: app.media_format, max_side: app.media_max, quality: app.media_quality },
            );
        }
    });
    if prog.total > 0 {
        card(ui, |ui| {
            if prog.running {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(format!("Converting {} of {}…", prog.done + 1, prog.total));
                });
                ui::bar(ui, ui.available_width(), prog.done as f32 / prog.total.max(1) as f32, ui::ACCENT);
            } else {
                let saved = if prog.saved_bytes >= 0 { format!("{} smaller", fmt_bytes(prog.saved_bytes as u64)) } else { format!("{} larger", fmt_bytes((-prog.saved_bytes) as u64)) };
                ui.label(title(format!("{} Converted {} image{}", ph::CHECK_CIRCLE, prog.outputs.len(), if prog.outputs.len() == 1 { "" } else { "s" })));
                ui.label(caption(format!("Saved next to the originals · {saved} in total")));
            }
            for e in prog.errors.iter().take(5) {
                ui.label(RichText::new(e).size(12.0).color(ui::ORANGE));
            }
        });
    }
}
