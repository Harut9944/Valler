//! More Utilities tools: Command bar, Quick toggles and Cleaning mode.

use crate::app::{Sub, ValleApp};
use crate::features::Tab;
use crate::quick_toggles;
use crate::tools_ui::{TOOLS, ago, refresh_ports};
use crate::ui::{self, caption, card, title};
use egui::{Color32, RichText, Ui};
use egui_phosphor::regular as ph;
use std::time::{Duration, Instant};

// ------------------------------------------------------------- command bar

enum CmdAction {
    Launch(crate::apps::AppEntry),
    Copy(String),
    Open(Sub),
    Tab(Tab),
    Note(usize),
    Web(String),
    KeepAwake,
    Lock,
    Settings,
}

struct CmdRow {
    icon: &'static str,
    title: String,
    detail: String,
    score: i32,
    action: CmdAction,
}

fn one_line(s: &str) -> String {
    let l = s.lines().find(|l| !l.trim().is_empty()).unwrap_or("").trim();
    if l.chars().count() > 70 { format!("{}…", l.chars().take(69).collect::<String>()) } else { l.to_string() }
}

fn command_rows(app: &ValleApp) -> Vec<CmdRow> {
    use crate::command_bar::{calc, convert, fmt_num, score};
    let q = app.cmd_query.trim();
    let ql = q.to_lowercase();
    let mut rows: Vec<CmdRow> = Vec::new();
    if let Some(v) = calc(q) {
        rows.push(CmdRow { icon: ph::CALCULATOR, title: format!("= {}", fmt_num(v)), detail: "Calculator · Enter copies the result".into(), score: 5000, action: CmdAction::Copy(fmt_num(v)) });
    }
    if let Some(c) = convert(q) {
        let value = c.split(" = ").nth(1).and_then(|r| r.split(' ').next()).unwrap_or("").to_string();
        rows.push(CmdRow { icon: ph::RULER, title: c, detail: "Unit conversion · Enter copies the result".into(), score: 5000, action: CmdAction::Copy(value) });
    }
    for t in TOOLS.iter().filter(|t| app.store.is_installed(t.feature) && t.sub != Sub::CommandBar) {
        if let Some(sc) = score(q, t.name) {
            rows.push(CmdRow { icon: t.icon, title: t.name.into(), detail: format!("Tool · {}", t.desc), score: sc + 50, action: CmdAction::Open(t.sub) });
        }
    }
    for tab in app.visible_tabs() {
        if tab == Tab::Utilities {
            continue;
        }
        if let Some(sc) = score(q, tab.title()) {
            rows.push(CmdRow { icon: tab.icon(), title: tab.title().into(), detail: "Panel".into(), score: sc + 40, action: CmdAction::Tab(tab) });
        }
    }
    if app.store.is_installed("keep_awake") {
        if let Some(sc) = score(q, "Keep awake now") {
            rows.push(CmdRow { icon: ph::COFFEE, title: "Keep awake now".into(), detail: "Action".into(), score: sc + 30, action: CmdAction::KeepAwake });
        }
    }
    if app.store.is_installed("quick_toggles") {
        if let Some(sc) = score(q, "Lock screen") {
            rows.push(CmdRow { icon: ph::LOCK, title: "Lock screen".into(), detail: "Action".into(), score: sc + 30, action: CmdAction::Lock });
        }
    }
    if let Some(sc) = score(q, "Settings") {
        rows.push(CmdRow { icon: ph::GEAR, title: "Settings".into(), detail: "Action".into(), score: sc + 20, action: CmdAction::Settings });
    }
    if !q.is_empty() {
        if let Some(apps) = app.apps.lock().ok().and_then(|a| a.clone()) {
            let mut found: Vec<(i32, crate::apps::AppEntry)> = apps.into_iter().filter_map(|a| score(q, &a.name).map(|s| (s, a))).collect();
            found.sort_by(|a, b| b.0.cmp(&a.0));
            for (sc, a) in found.into_iter().take(8) {
                let detail = if a.detail.is_empty() { "App".to_string() } else { format!("App · {}", a.detail) };
                rows.push(CmdRow { icon: ph::APP_WINDOW, title: a.name.clone(), detail, score: sc + 60, action: CmdAction::Launch(a) });
            }
        }
        let clip_now = app.store.clip_history.first().map(|e| e.text.clone()).unwrap_or_default();
        if app.store.is_installed("snippets") {
            for s in &app.store.snippets {
                let sc = score(q, &s.trigger).or_else(|| s.text.to_lowercase().contains(&ql).then_some(120));
                if let Some(sc) = sc {
                    let text = crate::snippets::expand(&s.text, &clip_now);
                    rows.push(CmdRow { icon: ph::TEXT_AA, title: s.trigger.clone(), detail: format!("Snippet · {}", one_line(&text)), score: sc, action: CmdAction::Copy(text) });
                }
            }
        }
        if app.store.is_installed("clipboard") {
            let mut n = 0;
            for e in &app.store.clip_history {
                if n >= 6 {
                    break;
                }
                if e.text.to_lowercase().contains(&ql) {
                    n += 1;
                    rows.push(CmdRow { icon: ph::CLIPBOARD_TEXT, title: one_line(&e.text), detail: format!("Clipboard · {}", ago(e.at)), score: 200 - n, action: CmdAction::Copy(e.text.clone()) });
                }
            }
        }
        if app.store.is_installed("scratchpad") {
            for (i, n) in app.store.notes.iter().enumerate() {
                if n.title.to_lowercase().contains(&ql) || n.body.to_lowercase().contains(&ql) {
                    rows.push(CmdRow { icon: ph::NOTE_PENCIL, title: n.title.clone(), detail: format!("Note · {}", one_line(&n.body)), score: 150, action: CmdAction::Note(i) });
                }
            }
        }
        rows.push(CmdRow { icon: ph::GLOBE, title: format!("Search the web for “{q}”"), detail: "Opens your browser".into(), score: -1000, action: CmdAction::Web(q.to_string()) });
    }
    rows.sort_by(|a, b| b.score.cmp(&a.score));
    rows.truncate(40);
    rows
}

pub fn command_bar_view(app: &mut ValleApp, ui: &mut Ui, ctx: &egui::Context) {
    let r = ui.add(
        egui::TextEdit::singleline(&mut app.cmd_query)
            .id(egui::Id::new("cmdbar-input"))
            .hint_text(format!("{}  An app, a tool, 12*7 or 10 km to mi", ph::MAGNIFYING_GLASS))
            .font(egui::FontId::proportional(16.0))
            .desired_width(f32::INFINITY)
            .margin(egui::vec2(12.0, 10.0)),
    );
    if app.cmd_focus {
        r.request_focus();
        app.cmd_focus = false;
    }
    if r.changed() {
        app.cmd_selected = 0;
    }
    let rows = command_rows(app);
    let (down, up, enter) = ctx.input(|i| (i.key_pressed(egui::Key::ArrowDown), i.key_pressed(egui::Key::ArrowUp), i.key_pressed(egui::Key::Enter)));
    if down && !rows.is_empty() {
        app.cmd_selected = (app.cmd_selected + 1).min(rows.len() - 1);
    }
    if up {
        app.cmd_selected = app.cmd_selected.saturating_sub(1);
    }
    app.cmd_selected = app.cmd_selected.min(rows.len().saturating_sub(1));
    let mut run: Option<usize> = (enter && !rows.is_empty() && (r.has_focus() || r.lost_focus())).then_some(app.cmd_selected);
    card(ui, |ui| {
        if rows.is_empty() {
            ui::empty_note(ui, ph::MAGNIFYING_GLASS, "Nothing found.");
        }
        for (i, row) in rows.iter().enumerate() {
            let sel = i == app.cmd_selected;
            let resp = egui::Frame::new()
                .fill(if sel { ui::ACCENT_SOFT } else { Color32::TRANSPARENT })
                .corner_radius(egui::CornerRadius::same(8))
                .inner_margin(egui::Margin::symmetric(8, 5))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.label(ui::icon(row.icon, 18.0, if sel { ui::ACCENT } else { ui::MUTED }));
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 0.0;
                            ui.add(egui::Label::new(RichText::new(&row.title).size(14.5)).truncate());
                            ui.add(egui::Label::new(RichText::new(&row.detail).size(11.5).color(ui::FAINT)).truncate());
                        });
                    });
                })
                .response;
            let resp = ui.interact(resp.rect, ui.id().with(("cmd", i)), egui::Sense::click());
            if resp.hovered() && ctx.input(|i| i.pointer.delta() != egui::Vec2::ZERO) {
                app.cmd_selected = i;
            }
            if resp.clicked() {
                run = Some(i);
            }
        }
    });
    ui.label(caption(format!("{} {} to move · Enter to run · Ctrl+K opens this from anywhere in Valler", ph::ARROW_UP, ph::ARROW_DOWN)));
    let Some(row) = run.and_then(|i| rows.into_iter().nth(i)) else { return };
    match row.action {
        CmdAction::Launch(a) => match crate::apps::launch(&a) {
            Ok(_) => app.toast(format!("Opening {}", a.name)),
            Err(e) => app.toast(format!("Couldn't open {}: {e}", a.name)),
        },
        CmdAction::Copy(t) => app.copy_text(ctx, t),
        CmdAction::Open(s) => {
            app.sub = Some(s);
            if s == Sub::Ports && app.ports.is_none() {
                refresh_ports(app);
            }
        }
        CmdAction::Tab(t) => {
            app.tab = t;
            app.sub = None;
        }
        CmdAction::Note(i) => {
            app.store.note_active = i;
            app.sub = Some(Sub::Scratchpad);
        }
        CmdAction::Web(q) => {
            let enc: String = url::form_urlencoded::byte_serialize(q.as_bytes()).collect();
            ctx.open_url(egui::OpenUrl::new_tab(format!("https://duckduckgo.com/?q={enc}")));
        }
        CmdAction::KeepAwake => {
            app.tab = Tab::KeepAwake;
            app.sub = None;
            if !app.awake.active() {
                app.awake.start(app.store.keep_awake_minutes, app.store.keep_display_on);
            }
        }
        CmdAction::Lock => {
            quick_toggles::spawn(app.qt_outcome.clone(), ctx.clone(), || {
                quick_toggles::lock_screen().map(|_| "Screen locked".to_string()).unwrap_or_else(|e| e)
            });
        }
        CmdAction::Settings => app.settings_open = true,
    }
    app.cmd_query.clear();
}

// ------------------------------------------------------------- quick toggles

fn toggle_tile(ui: &mut Ui, w: f32, glyph: &str, title_text: &str, caption_text: &str, active: bool) -> egui::Response {
    let id = ui.next_auto_id();
    let hovered = ui.ctx().read_response(id).is_some_and(|r| r.hovered());
    let inner = egui::Frame::new()
        .fill(if active { ui::ACCENT_SOFT } else if hovered { ui::CARD_HOVER } else { ui::CARD })
        .stroke(egui::Stroke::new(1.0, if active { ui::ACCENT } else { ui::BORDER }))
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(w - 24.0);
            ui.set_min_height(84.0);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 3.0;
                ui.label(ui::icon(glyph, 24.0, if active { ui::ACCENT } else { ui::TEXT }));
                ui.label(RichText::new(title_text).font(ui::bold(14.0)));
                ui.add(egui::Label::new(RichText::new(caption_text).size(11.5).color(ui::MUTED)).wrap());
            });
        });
    ui.interact(inner.response.rect, id, egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn quick_toggles_view(app: &mut ValleApp, ui: &mut Ui, ctx: &egui::Context) {
    if app.demo {
        if app.qt_checked.is_none() {
            app.qt_checked = Some(Instant::now());
            app.qt_dark = Some(true);
            app.qt_trash = Some(12);
        }
    } else if app.qt_checked.is_none_or(|t| t.elapsed() > Duration::from_secs(4)) {
        app.qt_checked = Some(Instant::now());
        app.qt_dark = quick_toggles::dark_mode();
        app.qt_trash = quick_toggles::trash_count();
    }
    let w = (ui.available_width() - 10.0) / 2.0;
    let dark = app.qt_dark.unwrap_or(false);
    let mut action: Option<u8> = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 10.0;
        let (t, c) = if dark { ("Switch to light", "The desktop uses a dark look") } else { ("Switch to dark", "The desktop uses a light look") };
        if toggle_tile(ui, w, if dark { ph::MOON } else { ph::SUN }, t, c, dark).clicked() {
            action = Some(0);
        }
        if toggle_tile(ui, w, ph::LOCK, "Lock screen", "Locks the computer right away", false).clicked() {
            action = Some(1);
        }
    });
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 10.0;
        if toggle_tile(ui, w, ph::MONITOR, "Display off", "Until you move the mouse or press a key", false).clicked() {
            action = Some(2);
        }
        if toggle_tile(ui, w, ph::SPARKLE, "Screen saver", "Starts the screen saver now", false).clicked() {
            action = Some(3);
        }
    });
    let trash_caption = match app.qt_trash {
        Some(0) => "The trash is already empty".to_string(),
        Some(n) => format!("{n} item{} in the trash", if n == 1 { "" } else { "s" }),
        None => "Deletes what is in the trash for good".into(),
    };
    ui.horizontal(|ui| {
        if toggle_tile(ui, w, ph::TRASH, "Empty trash", &trash_caption, false).clicked() && app.qt_trash != Some(0) {
            app.qt_confirm_trash = true;
        }
    });
    if app.qt_confirm_trash {
        card(ui, |ui| {
            ui.label(title("Empty the trash?"));
            ui.label(caption("Everything in the trash is deleted for good. This can't be undone."));
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new(RichText::new("Empty trash").color(Color32::WHITE)).fill(ui::RED)).clicked() {
                    action = Some(4);
                    app.qt_confirm_trash = false;
                }
                if ui::soft_button(ui, "Cancel").clicked() {
                    app.qt_confirm_trash = false;
                }
            });
        });
    }
    let Some(a) = action else { return };
    if app.demo {
        return;
    }
    let out = app.qt_outcome.clone();
    let ctx = ctx.clone();
    match a {
        0 => {
            app.qt_dark = Some(!dark);
            quick_toggles::spawn(out, ctx, move || match quick_toggles::set_dark_mode(!dark) {
                Ok(_) if dark => "Switched to light mode".into(),
                Ok(_) => "Switched to dark mode".into(),
                Err(e) => e,
            });
        }
        1 => quick_toggles::spawn(out, ctx, || quick_toggles::lock_screen().map(|_| "Screen locked".to_string()).unwrap_or_else(|e| e)),
        2 => quick_toggles::spawn(out, ctx, || quick_toggles::display_off().map(|_| "Display off".to_string()).unwrap_or_else(|e| e)),
        3 => quick_toggles::spawn(out, ctx, || quick_toggles::screen_saver().map(|_| "Screen saver started".to_string()).unwrap_or_else(|e| e)),
        _ => quick_toggles::spawn(out, ctx, || match quick_toggles::empty_trash() {
            Ok(n) if n > 0 => format!("Deleted {n} items from the trash"),
            Ok(_) => "Trash emptied".into(),
            Err(e) => format!("Couldn't empty the trash: {e}"),
        }),
    }
}

// ------------------------------------------------------------- cleaning mode

pub fn cleaning_view(app: &mut ValleApp, ui: &mut Ui) {
    card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(ui::icon(ph::KEYBOARD, 34.0, ui::ACCENT));
            ui.vertical(|ui| {
                ui.label(title("Clean your keyboard and screen"));
                ui.add(egui::Label::new(caption("Valler covers the screen and ignores typing until you press Esc several times in a row.")).wrap());
            });
        });
        ui::separator(ui);
        ui.horizontal(|ui| {
            ui.label("Presses of Esc to finish");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let mut n = app.store.cleaning_presses;
                if ui.add(egui::DragValue::new(&mut n).range(2..=10)).changed() {
                    app.store.cleaning_presses = n;
                    app.mark_dirty();
                }
            });
        });
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(ui.available_width() - 60.0);
                ui.label("Keep the screen visible");
                ui.add(egui::Label::new(caption("Dims the screen instead of turning it black.")).wrap());
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui::toggle(ui, &mut app.store.cleaning_keep_visible).changed() {
                    app.mark_dirty();
                }
            });
        });
        ui.add_space(6.0);
        let w = ui.available_width();
        let b = egui::Button::new(RichText::new(format!("{}  Start cleaning mode", ph::SPARKLE)).font(ui::bold(14.5)).color(Color32::WHITE))
            .fill(ui::ACCENT)
            .corner_radius(egui::CornerRadius::same(10))
            .min_size(egui::vec2(w, 36.0));
        if ui.add(b).clicked() && !app.demo {
            app.cleaning = true;
            app.cleaning_presses.clear();
        }
    });
    card(ui, |ui| {
        ui.add(egui::Label::new(caption("System shortcuts such as Alt+Tab, the Windows key or Ctrl+Alt+Del still reach the system, so avoid pressing them while wiping.")).wrap());
    });
}

/// The full-screen cover shown while cleaning mode is on.
pub fn cleaning_viewport(app: &mut ValleApp, ctx: &egui::Context) {
    let vid = egui::ViewportId::from_hash_of("valler-cleaning");
    let builder = egui::ViewportBuilder::default()
        .with_title("Valler cleaning mode")
        .with_fullscreen(true)
        .with_decorations(false)
        .with_transparent(app.store.cleaning_keep_visible)
        .with_always_on_top();
    let need = app.store.cleaning_presses.max(2) as usize;
    let visible = app.store.cleaning_keep_visible;
    let mut done = false;
    ctx.show_viewport_immediate(vid, builder, |ui, _| {
        let c = ui.ctx().clone();
        if c.input(|i| i.viewport().close_requested()) {
            c.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
        let events = c.input(|i| i.raw.events.clone());
        for e in events {
            if let egui::Event::Key { key, pressed: true, repeat: false, .. } = e {
                let now = Instant::now();
                if key == egui::Key::Escape {
                    app.cleaning_presses.retain(|t| now.duration_since(*t) < Duration::from_millis(1500));
                    app.cleaning_presses.push(now);
                    if app.cleaning_presses.len() >= need {
                        done = true;
                    }
                } else {
                    app.cleaning_presses.clear();
                }
            }
        }
        let fill = if visible { Color32::from_rgba_unmultiplied(0, 0, 0, 190) } else { Color32::BLACK };
        egui::CentralPanel::default().frame(egui::Frame::new().fill(fill)).show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space((ui.available_height() / 2.0 - 40.0).max(0.0));
                ui.label(ui::icon(ph::SPARKLE, 30.0, Color32::from_gray(90)));
                let left = need.saturating_sub(app.cleaning_presses.len());
                ui.label(RichText::new("Cleaning mode").font(ui::bold(16.0)).color(Color32::from_gray(120)));
                ui.label(RichText::new(format!("Press Esc {left} more time{} quickly to finish", if left == 1 { "" } else { "s" })).size(13.0).color(Color32::from_gray(90)));
            });
        });
        c.request_repaint_after(Duration::from_millis(250));
    });
    if done {
        app.cleaning = false;
        app.cleaning_presses.clear();
        app.toast("Cleaning mode finished");
    }
}

// ------------------------------------------------------------- timer

pub fn fmt_clock(d: Duration) -> String {
    let s = d.as_secs();
    if s >= 3600 { format!("{}:{:02}:{:02}", s / 3600, (s % 3600) / 60, s % 60) } else { format!("{:02}:{:02}", s / 60, s % 60) }
}

pub fn timer_view(app: &mut ValleApp, ui: &mut Ui) {
    let running = app.timer_end.is_some();
    let paused = app.timer_paused.is_some();
    let left = app.timer_end.map(|e| e.saturating_duration_since(Instant::now())).or(app.timer_paused);
    card(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(6.0);
            // Progress ring
            let size = 170.0;
            let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
            let c = rect.center();
            let radius = size / 2.0 - 8.0;
            ui.painter().circle_stroke(c, radius, egui::Stroke::new(8.0, ui::CONTROL));
            let frac = match left {
                Some(l) if app.timer_total.as_secs_f32() > 0.0 => l.as_secs_f32() / app.timer_total.as_secs_f32(),
                _ => 0.0,
            };
            if frac > 0.0 {
                let n = 64;
                let pts: Vec<egui::Pos2> = (0..=n)
                    .map(|i| {
                        let a = -std::f32::consts::FRAC_PI_2 + std::f32::consts::TAU * frac * i as f32 / n as f32;
                        c + egui::vec2(a.cos(), a.sin()) * radius
                    })
                    .collect();
                ui.painter().add(egui::Shape::line(pts, egui::Stroke::new(8.0, if paused { ui::YELLOW } else { ui::ACCENT })));
            }
            let text = match left {
                Some(l) => fmt_clock(l),
                None => fmt_clock(Duration::from_secs(app.timer_minutes as u64 * 60)),
            };
            ui.painter().text(c - egui::vec2(0.0, 6.0), egui::Align2::CENTER_CENTER, text, ui::bold(32.0), ui::TEXT);
            let sub = if paused { "Paused" } else if running { "Running" } else { "Ready" };
            ui.painter().text(c + egui::vec2(0.0, 26.0), egui::Align2::CENTER_CENTER, sub, egui::FontId::proportional(13.0), ui::MUTED);
            ui.add_space(4.0);
        });
        if !running && !paused {
            ui.horizontal_wrapped(|ui| {
                for m in [1u32, 5, 10, 15, 25, 45, 60] {
                    let sel = app.timer_minutes == m;
                    let b = egui::Button::new(RichText::new(format!("{m} min")).size(13.0).color(if sel { Color32::WHITE } else { ui::TEXT }))
                        .fill(if sel { ui::ACCENT } else { ui::CONTROL })
                        .corner_radius(egui::CornerRadius::same(8));
                    if ui.add(b).clicked() {
                        app.timer_minutes = m;
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.label(caption("Custom"));
                ui.add(egui::DragValue::new(&mut app.timer_minutes).range(1..=600).suffix(" min"));
                ui.add(egui::TextEdit::singleline(&mut app.timer_label).hint_text("Label (optional)").desired_width(f32::INFINITY));
            });
        } else if !app.timer_label.is_empty() {
            ui.vertical_centered(|ui| ui.label(title(&app.timer_label)));
        }
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let w = (ui.available_width() - 8.0) / 2.0;
            let primary = |t: String, fill: Color32| egui::Button::new(RichText::new(t).font(ui::bold(14.0)).color(Color32::WHITE)).fill(fill).corner_radius(egui::CornerRadius::same(10)).min_size(egui::vec2(w, 34.0));
            if !running && !paused {
                if ui.add(primary(format!("{}  Start", ph::PLAY), ui::ACCENT).min_size(egui::vec2(ui.available_width(), 34.0))).clicked() {
                    app.timer_total = Duration::from_secs(app.timer_minutes.max(1) as u64 * 60);
                    app.timer_end = Some(Instant::now() + app.timer_total);
                }
            } else {
                if running {
                    if ui.add(primary(format!("{}  Pause", ph::PAUSE), ui::CONTROL)).clicked() {
                        app.timer_paused = left;
                        app.timer_end = None;
                    }
                } else if ui.add(primary(format!("{}  Resume", ph::PLAY), ui::ACCENT)).clicked() {
                    app.timer_end = app.timer_paused.take().map(|l| Instant::now() + l);
                }
                if ui.add(primary(format!("{}  Cancel", ph::X), Color32::from_rgb(0x5a, 0x2a, 0x2a))).clicked() {
                    app.timer_end = None;
                    app.timer_paused = None;
                }
            }
        });
    });
    card(ui, |ui| {
        ui.add(egui::Label::new(caption("When time is up Valler shows a desktop notification, even if this window is minimized.")).wrap());
    });
}
