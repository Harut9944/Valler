//! Timer.

use crate::app::ValleApp;
use crate::ui::{self, caption, card, title};
use egui::{Color32, RichText, Ui};
use egui_phosphor::regular as ph;
use std::time::{Duration, Instant};

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

#[cfg(test)]
mod tests {
    use super::fmt_clock;
    use std::time::Duration;

    #[test]
    fn clock_format() {
        assert_eq!(fmt_clock(Duration::from_secs(65)), "01:05");
        assert_eq!(fmt_clock(Duration::from_secs(3725)), "1:02:05");
        assert_eq!(fmt_clock(Duration::ZERO), "00:00");
    }
}
