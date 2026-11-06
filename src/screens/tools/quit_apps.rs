//! Quit apps (and the process signalling the Port manager reuses).

use crate::app::ValleApp;
use crate::ui::{self, caption, card, fmt_bytes};
use egui::{RichText, Ui};
use egui_phosphor::regular as ph;
use sysinfo::{Pid, ProcessesToUpdate, Signal, System};
use super::*;

// ------------------------------------------------------------- quit apps

pub(crate) fn signal_pid(pid: u32, force: bool) -> bool {
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

pub(crate) fn quit_apps(app: &mut ValleApp, ui: &mut Ui) {
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
        procs.sort_by_key(|p| std::cmp::Reverse(p.mem));
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
