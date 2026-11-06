//! Port manager.

use crate::app::ValleApp;
use crate::services::ports;
use crate::ui::{self, card};
use egui::{RichText, Ui};
use egui_phosphor::regular as ph;
use super::*;

// ------------------------------------------------------------- ports

pub fn refresh_ports(app: &mut ValleApp) {
    app.ports = Some(if app.demo { Ok(ports::demo()) } else { ports::listening() });
}

pub(crate) fn ports_view(app: &mut ValleApp, ui: &mut Ui) {
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
