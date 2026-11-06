//! Utilities tab and its tools: Clipboard, Snippets, Scratchpad, Clean URL,
//! Quit apps, Port manager and Cleaner.

use crate::app::{Sub, ValleApp};
use crate::cleaner::{self, ScanState};
use crate::ports;
use crate::store::{NoteTab, Snippet};
use crate::ui::{self, caption, card, fmt_bytes, title};
use crate::{snippets, urlclean};
use egui::{Color32, RichText, Ui};
use egui_phosphor::regular as ph;
use sysinfo::{Pid, ProcessesToUpdate, Signal, System};

pub struct Tool {
    pub sub: Sub,
    pub feature: &'static str,
    pub icon: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
}

pub const TOOLS: &[Tool] = &[
    Tool { sub: Sub::CommandBar, feature: "command_bar", icon: ph::MAGNIFYING_GLASS, name: "Command bar", desc: "Search tools, clipboard, snippets and notes. Calculate and convert units." },
    Tool { sub: Sub::QuickToggles, feature: "quick_toggles", icon: ph::TOGGLE_LEFT, name: "Quick toggles", desc: "Dark mode, lock screen, display off, screen saver and empty the trash." },
    Tool { sub: Sub::Shelf, feature: "shelf", icon: ph::TRAY, name: "Shelf", desc: "Park files, text and links for a moment, then copy or move them where they belong." },
    Tool { sub: Sub::Clipboard, feature: "clipboard", icon: ph::CLIPBOARD_TEXT, name: "Clipboard", desc: "Stores copied text so you can reuse it later. Everything stays local and can be cleared anytime." },
    Tool { sub: Sub::Snippets, feature: "snippets", icon: ph::TEXT_AA, name: "Snippets", desc: "Saved text with date, time and clipboard variables, one click to copy." },
    Tool { sub: Sub::Scratchpad, feature: "scratchpad", icon: ph::NOTE_PENCIL, name: "Scratchpad", desc: "Autosaved notes in tabs." },
    Tool { sub: Sub::UrlClean, feature: "url_clean", icon: ph::LINK, name: "Clean URL", desc: "Removes tracking parameters from copied links." },
    Tool { sub: Sub::QuitApps, feature: "quit_apps", icon: ph::X_SQUARE, name: "Quit apps", desc: "See what is running and quit or force quit it." },
    Tool { sub: Sub::Ports, feature: "ports", icon: ph::PLUGS_CONNECTED, name: "Port manager", desc: "Find programs listening on network ports and stop them." },
    Tool { sub: Sub::Cleaner, feature: "cleaner", icon: ph::SPARKLE, name: "Cleaner", desc: "Caches, logs and temporary files." },
    Tool { sub: Sub::Packages, feature: "packages", icon: ph::PACKAGE, name: "Package manager", desc: "Search, install, update and remove apps without a terminal." },
    Tool { sub: Sub::Media, feature: "media", icon: ph::IMAGES, name: "Media", desc: "Convert, resize and compress images locally." },
    Tool { sub: Sub::AppUpdates, feature: "app_updates", icon: ph::ARROW_CIRCLE_UP, name: "App updates", desc: "Check apps from your stores and update them together." },
    Tool { sub: Sub::Uninstaller, feature: "uninstaller", icon: ph::TRASH, name: "Uninstaller", desc: "Removes apps you installed from an app store or package manager." },
    Tool { sub: Sub::Timer, feature: "timer", icon: ph::TIMER, name: "Timer", desc: "Countdown with a notification when time is up." },
    Tool { sub: Sub::CleaningMode, feature: "cleaning_mode", icon: ph::KEYBOARD, name: "Cleaning mode", desc: "Locks the keyboard so you can clean safely." },
];

pub fn utilities(app: &mut ValleApp, ui: &mut Ui) {
    for t in TOOLS {
        if !app.store.is_installed(t.feature) {
            continue;
        }
        let r = ui::clickable_card(ui, |ui| {
            ui::feature_row(ui, t.icon, t.name, t.desc, |ui| {
                ui.label(RichText::new(ph::CARET_RIGHT).color(ui::MUTED));
                if t.sub == Sub::CommandBar {
                    ui::key_chip(ui, if cfg!(target_os = "macos") { "⌘K" } else { "Ctrl+K" });
                }
                if t.sub == Sub::Timer {
                    if let Some(left) = app.timer_end.map(|e| e.saturating_duration_since(std::time::Instant::now())).or(app.timer_paused) {
                        ui.label(RichText::new(crate::tools_extra::fmt_clock(left)).font(ui::bold(13.0)).color(ui::ACCENT));
                    }
                }
                if t.sub == Sub::Shelf && !app.store.shelf_items.is_empty() {
                    ui.label(RichText::new(format!("{}", app.store.shelf_items.len())).font(ui::bold(12.5)).color(ui::ACCENT));
                }
                if t.sub == Sub::Clipboard {
                    ui.label(RichText::new(format!("{} saved", app.store.clip_history.len())).size(12.0).color(ui::FAINT));
                }
            });
        });
        if r.clicked() {
            app.sub = Some(t.sub);
            app.pkg_tab_forced = false;
            if t.sub == Sub::CommandBar {
                app.cmd_focus = true;
            }
            if t.sub == Sub::Ports && app.ports.is_none() {
                refresh_ports(app);
            }
        }
    }
}

pub fn sub_view(app: &mut ValleApp, ui: &mut Ui, ctx: &egui::Context, sub: Sub) {
    match sub {
        Sub::Clipboard => clipboard(app, ui, ctx),
        Sub::Snippets => snippets_view(app, ui, ctx),
        Sub::Scratchpad => scratchpad(app, ui),
        Sub::UrlClean => url_clean(app, ui, ctx),
        Sub::QuitApps => quit_apps(app, ui),
        Sub::Ports => ports_view(app, ui),
        Sub::Cleaner => cleaner_view(app, ui, ctx),
        Sub::CommandBar => crate::tools_extra::command_bar_view(app, ui, ctx),
        Sub::QuickToggles => crate::tools_extra::quick_toggles_view(app, ui, ctx),
        Sub::CleaningMode => crate::tools_extra::cleaning_view(app, ui),
        Sub::Timer => crate::tools_extra::timer_view(app, ui),
        Sub::Packages => crate::tools_more::packages_view(app, ui, ctx, None),
        Sub::Uninstaller => crate::tools_more::packages_view(app, ui, ctx, Some(0)),
        Sub::AppUpdates => crate::tools_more::packages_view(app, ui, ctx, Some(1)),
        Sub::Media => crate::tools_more::media_view(app, ui, ctx),
        Sub::Shelf => crate::shelf_ui::shelf_view(app, ui, ctx),
    }
}

fn search(ui: &mut Ui, text: &mut String, hint: &str) {
    ui.add(
        egui::TextEdit::singleline(text)
            .hint_text(format!("{}  {hint}", ph::MAGNIFYING_GLASS))
            .desired_width(f32::INFINITY)
            .margin(egui::vec2(10.0, 7.0)),
    );
}

pub fn ago(ts: i64) -> String {
    let d = (chrono::Utc::now().timestamp() - ts).max(0);
    match d {
        0..=59 => "just now".into(),
        60..=3599 => format!("{} min ago", d / 60),
        3600..=86_399 => format!("{} h ago", d / 3600),
        _ => format!("{} d ago", d / 86_400),
    }
}

// ------------------------------------------------------------- clipboard

fn clipboard(app: &mut ValleApp, ui: &mut Ui, ctx: &egui::Context) {
    if !app.store.is_on("clipboard") {
        card(ui, |ui| {
            ui.label(caption("Clipboard history is off. Turn it on to start saving what you copy."));
            if ui::primary_button(ui, "Turn on clipboard history").clicked() {
                app.store.set_on("clipboard", true);
                app.mark_dirty();
            }
        });
    }
    search(ui, &mut app.clip_search, "Search clipboard history");
    let q = app.clip_search.to_lowercase();
    let mut order: Vec<usize> = (0..app.store.clip_history.len())
        .filter(|i| q.is_empty() || app.store.clip_history[*i].text.to_lowercase().contains(&q))
        .collect();
    order.sort_by_key(|i| !app.store.clip_history[*i].pinned);
    if order.is_empty() {
        card(ui, |ui| ui::empty_note(ui, ph::CLIPBOARD, if q.is_empty() { "Nothing copied yet. Copy some text and it shows up here." } else { "No matches." }));
    }
    let mut action: Option<(usize, u8)> = None; // 0 copy, 1 pin, 2 delete
    for i in order.into_iter().take(150) {
        let e = &app.store.clip_history[i];
        let r = ui::clickable_card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width() - 70.0);
                    let preview: String = e.text.lines().take(3).collect::<Vec<_>>().join("\n");
                    let preview = if preview.chars().count() > 220 { format!("{}…", preview.chars().take(220).collect::<String>()) } else { preview };
                    ui.add(egui::Label::new(RichText::new(preview).size(13.5)).wrap());
                    ui.label(RichText::new(format!("{}{} · {} characters", if e.pinned { "📌 " } else { "" }, ago(e.at), e.text.chars().count())).size(11.5).color(ui::FAINT));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(RichText::new(ph::TRASH).color(ui::MUTED)).frame(false)).on_hover_text("Delete").clicked() {
                        action = Some((i, 2));
                    }
                    let pin = if e.pinned { ph::PUSH_PIN_SLASH } else { ph::PUSH_PIN };
                    if ui.add(egui::Button::new(RichText::new(pin).color(if e.pinned { ui::ACCENT } else { ui::MUTED })).frame(false)).on_hover_text(if e.pinned { "Unpin" } else { "Pin" }).clicked() {
                        action = Some((i, 1));
                    }
                });
            });
        });
        if r.clicked() && action.is_none() {
            action = Some((i, 0));
        }
    }
    if let Some((i, a)) = action {
        match a {
            0 => {
                let t = app.store.clip_history[i].text.clone();
                app.copy_text(ctx, t);
            }
            1 => {
                app.store.clip_history[i].pinned ^= true;
                app.mark_dirty();
            }
            _ => {
                app.store.clip_history.remove(i);
                app.mark_dirty();
            }
        }
    }
    ui.horizontal(|ui| {
        if ui::soft_button(ui, "Clear history (keep pinned)").clicked() {
            app.store.clip_history.retain(|e| e.pinned);
            app.mark_dirty();
        }
        if ui::soft_button(ui, "Empty system clipboard").clicked() {
            if let Some(w) = &app.clip {
                w.clear();
            }
            app.toast("Clipboard emptied");
        }
    });
    if app.store.is_installed("paste_plain") && ui::soft_button(ui, format!("{} Make clipboard plain text", ph::TEXT_T)).on_hover_text("Removes formatting so the next paste is plain text").clicked() {
        make_plain(app, ctx);
    }
}

// ------------------------------------------------------------- snippets

fn snippets_view(app: &mut ValleApp, ui: &mut Ui, ctx: &egui::Context) {
    card(ui, |ui| {
        ui.label(caption("Variables: {{date}}, {{time}}, {{datetime}}, {{clipboard}}, {{name}}. A format after a colon picks the look, like {{date:%d.%m.%Y}}."));
    });
    let clip_now = app.store.clip_history.first().map(|e| e.text.clone()).unwrap_or_default();
    let mut action: Option<(usize, u8)> = None;
    for (i, s) in app.store.snippets.iter().enumerate() {
        card(ui, |ui| {
            ui.horizontal(|ui| {
                ui::key_chip(ui, &s.trigger);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(RichText::new(ph::TRASH).color(ui::MUTED)).frame(false)).on_hover_text("Delete").clicked() {
                        action = Some((i, 2));
                    }
                    if ui.add(egui::Button::new(RichText::new(ph::PENCIL_SIMPLE).color(ui::MUTED)).frame(false)).on_hover_text("Edit").clicked() {
                        action = Some((i, 1));
                    }
                    if ui::primary_button(ui, format!("{} Copy", ph::COPY)).clicked() {
                        action = Some((i, 0));
                    }
                });
            });
            let preview = snippets::expand(&s.text, &clip_now);
            ui.add(egui::Label::new(RichText::new(preview).size(13.5)).wrap());
        });
    }
    if let Some((i, a)) = action {
        match a {
            0 => {
                let t = snippets::expand(&app.store.snippets[i].text, &clip_now);
                app.copy_text(ctx, t);
            }
            1 => {
                app.snip_trigger = app.store.snippets[i].trigger.clone();
                app.snip_text = app.store.snippets[i].text.clone();
                app.snip_editing = Some(i);
            }
            _ => {
                app.store.snippets.remove(i);
                app.snip_editing = None;
                app.mark_dirty();
            }
        }
    }
    card(ui, |ui| {
        ui.label(title(if app.snip_editing.is_some() { "Edit snippet" } else { "New snippet" }));
        ui.add(egui::TextEdit::singleline(&mut app.snip_trigger).hint_text("Name or trigger, like ;addr").desired_width(f32::INFINITY));
        ui.add(egui::TextEdit::multiline(&mut app.snip_text).hint_text("Text to insert").desired_rows(3).desired_width(f32::INFINITY));
        ui.horizontal(|ui| {
            let ok = !app.snip_trigger.trim().is_empty() && !app.snip_text.is_empty();
            if ui.add_enabled(ok, egui::Button::new(RichText::new("Save").color(Color32::WHITE)).fill(ui::ACCENT)).clicked() {
                let sn = Snippet { trigger: app.snip_trigger.trim().to_string(), text: app.snip_text.clone() };
                match app.snip_editing.take() {
                    Some(i) if i < app.store.snippets.len() => app.store.snippets[i] = sn,
                    _ => app.store.snippets.push(sn),
                }
                app.snip_trigger.clear();
                app.snip_text.clear();
                app.mark_dirty();
            }
            if app.snip_editing.is_some() && ui::soft_button(ui, "Cancel").clicked() {
                app.snip_editing = None;
                app.snip_trigger.clear();
                app.snip_text.clear();
            }
        });
    });
}

// ------------------------------------------------------------- scratchpad

fn scratchpad(app: &mut ValleApp, ui: &mut Ui) {
    if app.store.notes.is_empty() {
        app.store.notes.push(NoteTab { title: "Notes".into(), body: String::new() });
    }
    app.store.note_active = app.store.note_active.min(app.store.notes.len() - 1);
    ui.horizontal_wrapped(|ui| {
        for i in 0..app.store.notes.len() {
            let sel = i == app.store.note_active;
            let t = app.store.notes[i].title.clone();
            let b = egui::Button::new(RichText::new(if t.is_empty() { "Untitled".into() } else { t }).size(13.0).color(if sel { Color32::WHITE } else { ui::TEXT }))
                .fill(if sel { ui::ACCENT } else { ui::CONTROL });
            if ui.add(b).clicked() {
                app.store.note_active = i;
                app.mark_dirty();
            }
        }
        if ui.add(egui::Button::new(RichText::new(ph::PLUS)).fill(ui::CONTROL)).on_hover_text("New tab").clicked() {
            app.store.notes.push(NoteTab { title: format!("Note {}", app.store.notes.len() + 1), body: String::new() });
            app.store.note_active = app.store.notes.len() - 1;
            app.mark_dirty();
        }
    });
    let idx = app.store.note_active;
    let mut changed = false;
    card(ui, |ui| {
        let n = &mut app.store.notes[idx];
        changed |= ui.add(egui::TextEdit::singleline(&mut n.title).font(ui::bold(15.0)).frame(egui::Frame::NONE).desired_width(f32::INFINITY)).changed();
        ui::separator(ui);
        changed |= ui
            .add(egui::TextEdit::multiline(&mut n.body).hint_text("Write anything. It saves by itself.").frame(egui::Frame::NONE).desired_rows(16).desired_width(f32::INFINITY))
            .changed();
        ui.label(caption(format!("{} words · {} characters", n.body.split_whitespace().count(), n.body.chars().count())));
    });
    if changed {
        app.mark_dirty();
    }
    ui.horizontal(|ui| {
        if ui::soft_button(ui, format!("{} Export to Documents", ph::EXPORT)).clicked() {
            let n = &app.store.notes[idx];
            let dir = dirs::document_dir().or_else(dirs::home_dir).unwrap_or_default();
            let safe: String = n.title.chars().map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' { c } else { '_' }).collect();
            let path = dir.join(format!("{}.md", if safe.trim().is_empty() { "Note".to_string() } else { safe.trim().to_string() }));
            match std::fs::write(&path, &n.body) {
                Ok(_) => app.toast(format!("Saved {}", path.display())),
                Err(e) => app.toast(format!("Could not save: {e}")),
            }
        }
        if app.store.notes.len() > 1 && ui::soft_button(ui, format!("{} Delete tab", ph::TRASH)).clicked() {
            app.store.notes.remove(idx);
            app.store.note_active = 0;
            app.mark_dirty();
        }
    });
}

// ------------------------------------------------------------- clean url

fn url_clean(app: &mut ValleApp, ui: &mut Ui, ctx: &egui::Context) {
    card(ui, |ui| {
        ui.label(title("Clean a link"));
        let r = ui.add(egui::TextEdit::singleline(&mut app.url_input).hint_text("Paste a link").desired_width(f32::INFINITY).margin(egui::vec2(10.0, 7.0)));
        let enter = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        ui.horizontal(|ui| {
            if ui::primary_button(ui, "Clean").clicked() || enter {
                run_clean(app);
            }
            if ui::soft_button(ui, "Clean the link on my clipboard").clicked() {
                let latest = arboard::Clipboard::new().ok().and_then(|mut c| c.get_text().ok()).or_else(|| app.store.clip_history.first().map(|e| e.text.clone()));
                app.url_input = latest.unwrap_or_default();
                run_clean(app);
                if let Some((u, removed)) = app.url_result.clone() {
                    if !removed.is_empty() {
                        app.copy_text(ctx, u);
                    }
                }
            }
        });
        if let Some(e) = &app.url_error {
            ui.label(RichText::new(e).color(ui::ORANGE).size(12.5));
        }
        if let Some((u, removed)) = app.url_result.clone() {
            ui::separator(ui);
            ui.add(egui::Label::new(RichText::new(&u).size(13.0).color(ui::ACCENT)).wrap());
            if removed.is_empty() {
                ui.label(caption("Nothing to remove; this link was already clean."));
            } else {
                ui.label(caption(format!("Removed: {}", removed.join(", "))));
            }
            if ui::soft_button(ui, format!("{} Copy clean link", ph::COPY)).clicked() {
                app.copy_text(ctx, u);
            }
        }
    });
    card(ui, |ui| {
        let mut v = app.store.is_on("url_clean");
        ui::feature_row(ui, ph::MAGIC_WAND, "Clean copied links automatically", "Every link you copy loses its tracking parameters.", |ui| {
            ui::toggle(ui, &mut v);
        });
        if v != app.store.is_on("url_clean") {
            app.store.set_on("url_clean", v);
            app.mark_dirty();
        }
        ui.label(caption("Extra parameters to remove (comma separated)"));
        if ui.add(egui::TextEdit::singleline(&mut app.store.url_extra_params).hint_text("ref, source").desired_width(f32::INFINITY)).changed() {
            app.mark_dirty();
        }
    });
}

fn run_clean(app: &mut ValleApp) {
    match urlclean::clean(&app.url_input, &app.store.url_extra_params) {
        Some(r) => {
            app.url_result = Some((r.url, r.removed));
            app.url_error = None;
        }
        None => {
            app.url_result = None;
            app.url_error = Some("That doesn't look like a single web link.".into());
        }
    }
}

// ------------------------------------------------------------- quit apps

fn signal_pid(pid: u32, force: bool) -> bool {
    let Ok(mut sys) = System::new() else { return false };
    let p = Pid::from_u32(pid);
    sys.refresh_processes(ProcessesToUpdate::Some(&[p]), true);
    let Some(proc_) = sys.process(p) else { return false };
    if force {
        proc_.kill()
    } else {
        proc_.kill_with(Signal::Term).unwrap_or_else(|| proc_.kill())
    }
}

fn quit_apps(app: &mut ValleApp, ui: &mut Ui) {
    search(ui, &mut app.proc_filter, "Filter programs");
    ui.horizontal(|ui| {
        ui.label(caption("Sort by"));
        if ui.selectable_label(!app.proc_sort_mem, "Processor").clicked() {
            app.proc_sort_mem = false;
        }
        if ui.selectable_label(app.proc_sort_mem, "Memory").clicked() {
            app.proc_sort_mem = true;
        }
    });
    let mut procs = app.snap.procs.clone();
    if app.proc_sort_mem {
        procs.sort_by(|a, b| b.mem.cmp(&a.mem));
    }
    let q = app.proc_filter.to_lowercase();
    let me = std::process::id();
    let mut msg = None;
    card(ui, |ui| {
        let list: Vec<_> = procs.iter().filter(|p| p.pid != me && (q.is_empty() || p.name.to_lowercase().contains(&q))).take(60).collect();
        if list.is_empty() {
            ui::empty_note(ui, ph::MAGNIFYING_GLASS, "No matching programs.");
        }
        for p in list {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width() - 120.0);
                    ui.spacing_mut().item_spacing.y = 0.0;
                    ui.add(egui::Label::new(RichText::new(&p.name).size(14.0)).truncate());
                    ui.label(RichText::new(format!("{:.1}% · {} · pid {}", p.cpu, fmt_bytes(p.mem), p.pid)).size(11.5).color(ui::FAINT));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(RichText::new("Force").size(12.5).color(ui::RED)).fill(ui::CONTROL)).on_hover_text("Stop it immediately; unsaved work is lost").clicked() {
                        msg = Some(if signal_pid(p.pid, true) { format!("Force quit {}", p.name) } else { format!("Couldn't stop {}", p.name) });
                    }
                    if ui.add(egui::Button::new(RichText::new("Quit").size(12.5)).fill(ui::CONTROL)).on_hover_text("Ask it to close").clicked() {
                        msg = Some(if signal_pid(p.pid, false) { format!("Asked {} to quit", p.name) } else { format!("Couldn't quit {}", p.name) });
                    }
                });
            });
        }
    });
    if let Some(m) = msg {
        app.toast(m);
    }
}

// ------------------------------------------------------------- ports

pub fn refresh_ports(app: &mut ValleApp) {
    app.ports = Some(if app.demo { Ok(ports::demo()) } else { ports::listening() });
}

fn ports_view(app: &mut ValleApp, ui: &mut Ui) {
    ui.horizontal(|ui| {
        if ui::soft_button(ui, format!("{} Refresh", ph::ARROWS_CLOCKWISE)).clicked() {
            refresh_ports(app);
        }
        ui.add(egui::TextEdit::singleline(&mut app.port_filter).hint_text("Filter by port or program").desired_width(f32::INFINITY));
    });
    let names: std::collections::HashMap<u32, String> = app.snap.procs.iter().map(|p| (p.pid, p.name.clone())).collect();
    let mut msg = None;
    match app.ports.clone() {
        None => {}
        Some(Err(e)) => {
            card(ui, |ui| ui.label(RichText::new(format!("Couldn't read network ports: {e}")).color(ui::RED)));
        }
        Some(Ok(list)) => {
            let q = app.port_filter.to_lowercase();
            card(ui, |ui| {
                let mut shown = 0;
                for e in &list {
                    let pname = e.pids.first().and_then(|p| names.get(p)).cloned().unwrap_or_else(|| if e.pids.is_empty() { "System or another user".into() } else { "Unknown".into() });
                    if !q.is_empty() && !e.port.to_string().contains(&q) && !pname.to_lowercase().contains(&q) {
                        continue;
                    }
                    shown += 1;
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("{}", e.port)).font(ui::bold(15.0)).color(ui::ACCENT));
                        ui.vertical(|ui| {
                            ui.set_width(ui.available_width() - 70.0);
                            ui.spacing_mut().item_spacing.y = 0.0;
                            ui.add(egui::Label::new(RichText::new(&pname).size(14.0)).truncate());
                            ui.label(RichText::new(format!("{} · {}{}", e.proto, e.addr, e.pids.first().map(|p| format!(" · pid {p}")).unwrap_or_default())).size(11.5).color(ui::FAINT));
                        });
                        if let Some(pid) = e.pids.first().copied() {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.add(egui::Button::new(RichText::new("Stop").size(12.5).color(ui::RED)).fill(ui::CONTROL)).clicked() {
                                    msg = Some(if signal_pid(pid, false) { format!("Stopped {pname}") } else { format!("Couldn't stop {pname}") });
                                }
                            });
                        }
                    });
                }
                if shown == 0 {
                    ui::empty_note(ui, ph::PLUGS, "Nothing is listening.");
                }
            });
        }
    }
    if let Some(m) = msg {
        app.toast(m);
        refresh_ports(app);
    }
}

// ------------------------------------------------------------- cleaner

fn cleaner_view(app: &mut ValleApp, ui: &mut Ui, ctx: &egui::Context) {
    if app.demo {
        if let Ok(mut s) = app.cleaner.lock() {
            if matches!(*s, ScanState::Idle) {
                *s = ScanState::Done(cleaner::demo());
            }
        }
    }
    let state = app.cleaner.lock().map(|s| s.clone()).unwrap_or_default();
    match state {
        ScanState::Idle => {
            card(ui, |ui| {
                ui::empty_note(ui, ph::SPARKLE, "Look for caches, logs and temporary files you can safely remove. Everything goes to the trash, so you can put it back.");
                if ui::primary_button(ui, "Scan").clicked() {
                    cleaner::scan(app.cleaner.clone(), ctx.clone());
                }
            });
        }
        ScanState::Scanning => {
            card(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Working…");
                });
            });
        }
        ScanState::Cleaned { freed, failed } => {
            card(ui, |ui| {
                ui.label(title(format!("{} Moved {} to the trash", ph::CHECK_CIRCLE, fmt_bytes(freed))));
                if failed > 0 {
                    ui.label(caption(format!("{failed} items were in use or protected and were skipped.")));
                }
                if ui::soft_button(ui, "Scan again").clicked() {
                    cleaner::scan(app.cleaner.clone(), ctx.clone());
                }
            });
        }
        ScanState::Done(mut items) => {
            let mut changed = false;
            let total: u64 = items.iter().filter(|i| i.selected).map(|i| i.size).sum();
            card(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(title(format!("{} selected", fmt_bytes(total))));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add_enabled(total > 0, egui::Button::new(RichText::new("Move to trash").color(Color32::WHITE)).fill(ui::ACCENT)).clicked() {
                            cleaner::clean(app.cleaner.clone(), items.clone(), ctx.clone());
                        }
                        if ui::soft_button(ui, "Rescan").clicked() {
                            cleaner::scan(app.cleaner.clone(), ctx.clone());
                        }
                    });
                });
                if items.is_empty() {
                    ui::empty_note(ui, ph::CHECK_CIRCLE, "Nothing worth cleaning.");
                }
                let mut all = !items.is_empty() && items.iter().all(|i| i.selected);
                if !items.is_empty() && ui.checkbox(&mut all, "Select all").changed() {
                    for i in items.iter_mut() {
                        i.selected = all;
                    }
                    changed = true;
                }
                for it in items.iter_mut().take(200) {
                    ui.horizontal(|ui| {
                        changed |= ui.checkbox(&mut it.selected, "").changed();
                        ui.vertical(|ui| {
                            ui.set_width(ui.available_width() - 80.0);
                            ui.spacing_mut().item_spacing.y = 0.0;
                            let name = it.path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                            ui.add(egui::Label::new(RichText::new(name).size(14.0)).truncate());
                            ui.add(egui::Label::new(RichText::new(format!("{} · {}", it.category, it.path.display())).size(11.5).color(ui::FAINT)).truncate());
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(fmt_bytes(it.size)).color(ui::MUTED));
                        });
                    });
                }
            });
            if changed {
                if let Ok(mut s) = app.cleaner.lock() {
                    *s = ScanState::Done(items);
                }
            }
        }
    }
}

/// Re-puts the clipboard text without formatting (Paste as plain text).
pub fn make_plain(app: &mut ValleApp, ctx: &egui::Context) {
    match arboard::Clipboard::new().and_then(|mut c| c.get_text()) {
        Ok(t) if !t.is_empty() => {
            app.copy_text(ctx, t);
            app.toast("Clipboard is now plain text");
        }
        _ => app.toast("There's no text on the clipboard"),
    }
}
