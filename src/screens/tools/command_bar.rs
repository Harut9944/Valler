//! Command bar: one field for tools, apps, clipboard, snippets, notes,
//! sums and unit conversions.

use crate::app::{Sub, ValleApp};
use crate::core::features::Tab;
use crate::services::quick_toggles;
use crate::screens::tools::{TOOLS, ago, refresh_ports};
use crate::ui::{self, caption, card};
use egui::{Color32, RichText, Ui};
use egui_phosphor::regular as ph;

// ------------------------------------------------------------- command bar

enum CmdAction {
    Plain,
    Launch(crate::services::apps::AppEntry),
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
    /// A colour sample drawn instead of the icon (colour answers).
    swatch: Option<[u8; 3]>,
    icon: &'static str,
    title: String,
    detail: String,
    score: i32,
    action: CmdAction,
}

pub(crate) fn one_line(s: &str) -> String {
    let l = s.lines().find(|l| !l.trim().is_empty()).unwrap_or("").trim();
    if l.chars().count() > 70 { format!("{}…", l.chars().take(69).collect::<String>()) } else { l.to_string() }
}

fn command_rows(app: &ValleApp) -> Vec<CmdRow> {
    use crate::services::command_bar::{calc, color, convert, date_answer, emoji, fmt_num, score};
    let q = app.cmd_query.trim();
    let ql = q.to_lowercase();
    let mut rows: Vec<CmdRow> = Vec::new();
    if let Some(v) = calc(q) {
        rows.push(CmdRow { swatch: None, icon: ph::CALCULATOR, title: format!("= {}", fmt_num(v)), detail: "Calculator · Enter copies the result".into(), score: 5000, action: CmdAction::Copy(fmt_num(v)) });
    }
    if let Some(c) = convert(q) {
        let value = c.split(" = ").nth(1).and_then(|r| r.split(' ').next()).unwrap_or("").to_string();
        rows.push(CmdRow { swatch: None, icon: ph::RULER, title: c, detail: "Unit conversion · Enter copies the result".into(), score: 5000, action: CmdAction::Copy(value) });
    }
    if let Some(c) = color(q) {
        for (i, (text, label)) in [(c.hex.clone(), "HEX"), (c.rgb_text.clone(), "RGB"), (c.hsl_text.clone(), "HSL")].into_iter().enumerate() {
            rows.push(CmdRow { swatch: Some(c.rgb), icon: ph::PALETTE, title: text.clone(), detail: format!("Colour · {label} · Enter copies"), score: 5000 - i as i32, action: CmdAction::Copy(text) });
        }
    }
    if let Some(d) = date_answer(q) {
        rows.push(CmdRow { swatch: None, icon: ph::CALENDAR, title: d.clone(), detail: "Date · Enter copies".into(), score: 5000, action: CmdAction::Copy(d) });
    }
    for (i, (e, name)) in emoji(q).into_iter().enumerate() {
        rows.push(CmdRow { swatch: None, icon: ph::SMILEY, title: format!("{e}   {name}"), detail: "Emoji · Enter copies".into(), score: 4900 - i as i32, action: CmdAction::Copy(e.to_string()) });
    }
    for t in TOOLS.iter().filter(|t| app.store.is_installed(t.feature) && t.sub != Sub::CommandBar) {
        if let Some(sc) = score(q, t.name) {
            rows.push(CmdRow { swatch: None, icon: t.icon, title: t.name.into(), detail: format!("Tool · {}", t.desc), score: sc + 50, action: CmdAction::Open(t.sub) });
        }
    }
    for tab in app.visible_tabs() {
        if tab == Tab::Utilities {
            continue;
        }
        if let Some(sc) = score(q, tab.title()) {
            rows.push(CmdRow { swatch: None, icon: tab.icon(), title: tab.title().into(), detail: "Panel".into(), score: sc + 40, action: CmdAction::Tab(tab) });
        }
    }
    if app.store.is_installed("keep_awake")
        && let Some(sc) = score(q, "Keep awake now") {
            rows.push(CmdRow { swatch: None, icon: ph::COFFEE, title: "Keep awake now".into(), detail: "Action".into(), score: sc + 30, action: CmdAction::KeepAwake });
        }
    if app.store.is_installed("quick_toggles")
        && let Some(sc) = score(q, "Lock screen") {
            rows.push(CmdRow { swatch: None, icon: ph::LOCK, title: "Lock screen".into(), detail: "Action".into(), score: sc + 30, action: CmdAction::Lock });
        }
    if app.store.is_installed("paste_plain")
        && let Some(sc) = score(q, "Paste as plain text") {
            rows.push(CmdRow { swatch: None, icon: ph::TEXT_T, title: "Paste as plain text".into(), detail: "Action · removes formatting from the clipboard".into(), score: sc + 30, action: CmdAction::Plain });
        }
    if let Some(sc) = score(q, "Settings") {
        rows.push(CmdRow { swatch: None, icon: ph::GEAR, title: "Settings".into(), detail: "Action".into(), score: sc + 20, action: CmdAction::Settings });
    }
    if !q.is_empty() {
        if let Some(apps) = app.apps.lock().ok().and_then(|a| a.clone()) {
            let mut found: Vec<(i32, crate::services::apps::AppEntry)> = apps.into_iter().filter_map(|a| score(q, &a.name).map(|s| (s, a))).collect();
            found.sort_by_key(|f| std::cmp::Reverse(f.0));
            for (sc, a) in found.into_iter().take(8) {
                let detail = if a.detail.is_empty() { "App".to_string() } else { format!("App · {}", a.detail) };
                rows.push(CmdRow { swatch: None, icon: ph::APP_WINDOW, title: a.name.clone(), detail, score: sc + 60, action: CmdAction::Launch(a) });
            }
        }
        let clip_now = app.store.clip_history.first().map(|e| e.text.clone()).unwrap_or_default();
        if app.store.is_installed("snippets") {
            for s in &app.store.snippets {
                let sc = score(q, &s.trigger).or_else(|| s.text.to_lowercase().contains(&ql).then_some(120));
                if let Some(sc) = sc {
                    let text = crate::services::snippets::expand(&s.text, &clip_now);
                    rows.push(CmdRow { swatch: None, icon: ph::TEXT_AA, title: s.trigger.clone(), detail: format!("Snippet · {}", one_line(&text)), score: sc, action: CmdAction::Copy(text) });
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
                    rows.push(CmdRow { swatch: None, icon: ph::CLIPBOARD_TEXT, title: one_line(&e.text), detail: format!("Clipboard · {}", ago(e.at)), score: 200 - n, action: CmdAction::Copy(e.text.clone()) });
                }
            }
        }
        if app.store.is_installed("scratchpad") {
            for (i, n) in app.store.notes.iter().enumerate() {
                if n.title.to_lowercase().contains(&ql) || n.body.to_lowercase().contains(&ql) {
                    rows.push(CmdRow { swatch: None, icon: ph::NOTE_PENCIL, title: n.title.clone(), detail: format!("Note · {}", one_line(&n.body)), score: 150, action: CmdAction::Note(i) });
                }
            }
        }
        rows.push(CmdRow { swatch: None, icon: ph::GLOBE, title: format!("Search the web for “{q}”"), detail: "Opens your browser".into(), score: -1000, action: CmdAction::Web(q.to_string()) });
    }
    rows.sort_by_key(|r| std::cmp::Reverse(r.score));
    rows.truncate(40);
    rows
}

pub fn command_bar_view(app: &mut ValleApp, ui: &mut Ui, ctx: &egui::Context) {
    let r = ui.add(
        egui::TextEdit::singleline(&mut app.cmd_query)
            .id(egui::Id::new("cmdbar-input"))
            .hint_text(format!("{}  An app, a tool, 12*7, 10 km to mi, #ff8800, tomorrow…", ph::MAGNIFYING_GLASS))
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
                        match row.swatch {
                            Some([r, g, b]) => {
                                let (rect, _) = ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::hover());
                                ui.painter().circle(rect.center(), 8.0, Color32::from_rgb(r, g, b), egui::Stroke::new(1.0, ui::BORDER));
                            }
                            None => {
                                ui.label(ui::icon(row.icon, 18.0, if sel { ui::ACCENT } else { ui::MUTED }));
                            }
                        }
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
        CmdAction::Plain => crate::screens::tools::make_plain(app, ctx),
        CmdAction::Launch(a) => match crate::services::apps::launch(&a) {
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

#[cfg(test)]
mod feature_tests {
    use super::*;
    use crate::smoke_tests::demo_app;

    fn top(app: &mut ValleApp, q: &str) -> Vec<String> {
        app.cmd_query = q.into();
        command_rows(app).into_iter().map(|r| r.title).collect()
    }

    #[test]
    fn command_bar_calculates_converts_and_finds_tools() {
        let (_ctx, mut app) = demo_app();
        assert_eq!(top(&mut app, "12*7")[0], "= 84");
        assert!(top(&mut app, "10 km to m")[0].starts_with("10 km = 10000 m"));
        assert_eq!(top(&mut app, "shelf")[0], "Shelf");
        assert_eq!(top(&mut app, "timer")[0], "Timer");
        // Web search is always offered last for a typed query.
        assert!(top(&mut app, "anything").last().unwrap().starts_with("Search the web"));
    }

    #[test]
    fn command_bar_searches_clipboard_snippets_and_notes() {
        let (_ctx, mut app) = demo_app();
        assert!(top(&mut app, "meeting").iter().any(|t| t.contains("Meeting moved")), "clipboard history");
        assert!(top(&mut app, ";sig").contains(&";sig".to_string()), "snippets");
        app.store.notes[0].body = "groceries: apples".into();
        assert!(top(&mut app, "apples").contains(&"Plan".to_string()), "scratchpad notes");
    }

    #[test]
    fn command_bar_answers_colours_dates_and_emoji() {
        let (_ctx, mut app) = demo_app();
        let rows = top(&mut app, "#ff8800");
        assert_eq!(&rows[..3], &["#FF8800", "rgb(255, 136, 0)", "hsl(32, 100%, 50%)"]);
        assert!(top(&mut app, "tomorrow")[0].contains(", "), "a full date like Saturday, 7 November 2026");
        assert!(top(&mut app, "emoji rocket")[0].starts_with("🚀"));
    }

    #[test]
    fn command_bar_hides_uninstalled_tools() {
        let (_ctx, mut app) = demo_app();
        app.store.installed.remove("timer");
        assert!(!top(&mut app, "timer").contains(&"Timer".to_string()));
        app.cmd_query.clear();
        assert!(!command_rows(&app).is_empty(), "an empty query lists tools to browse");
    }

}
