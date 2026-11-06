//! The eframe entry points: per-frame logic (runs even while hidden), drawing,
//! saving on exit and the transparent clear colour.

use crate::core::features::Tab;
use crate::ui;
use egui::Ui;
use std::time::{Duration, Instant};
use super::*;

impl eframe::App for ValleApp {
    /// Runs even while the window is hidden, so alerts keep working.
    fn logic(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(s) = self.mon.lock() {
            self.snap = s.clone();
        }
        self.tray_tick(_ctx);
        if !self.demo {
            self.alerts.evaluate(&self.snap, &self.store);
            self.automate_keep_awake();
        }
        // Timer finishes even while the window is minimized.
        if let Some(end) = self.timer_end {
            if Instant::now() >= end {
                self.timer_end = None;
                let label = if self.timer_label.trim().is_empty() { "Timer".to_string() } else { self.timer_label.trim().to_string() };
                crate::services::alerts::notify(&format!("{label} is done"), &format!("{} minutes are up.", self.timer_total.as_secs() / 60));
                self.toast(format!("{label} is done"));
            } else {
                _ctx.request_repaint_after(Duration::from_millis(500));
            }
        }
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        // Ctrl+K (Cmd+K on macOS) opens the command bar from anywhere in the panel.
        if self.store.is_installed("command_bar")
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::K))
        {
            self.tab = Tab::Utilities;
            self.sub = Some(Sub::CommandBar);
            self.cmd_focus = true;
        }
        if self.sub == Some(Sub::CommandBar) && !self.apps_scanned {
            self.apps_scanned = true;
            crate::services::apps::scan(self.apps.clone(), ctx.clone());
        }
        if let Some(msg) = self.qt_outcome.lock().ok().and_then(|mut o| o.take()) {
            self.toast(msg);
            self.qt_checked = None;
        }
        if self.cleaning {
            crate::screens::tools::cleaning_viewport(self, &ctx);
        }
        if self.shelf_floating && self.store.is_installed("shelf") {
            crate::screens::tools::shelf::floating(self, &ctx);
        }
        self.awake.tick();
        self.drain_clipboard();
        if self.tab == Tab::Mixer && self.sub.is_none() {
            self.poll_mixer(&ctx);
        }
        if self.awake.active() || self.awake.demo_active() {
            ctx.request_repaint_after(Duration::from_secs(1));
        }

        let settings_inline = self.demo && self.opts.settings;
        ui::window_surface(ui);
        if self.demo && self.opts.tab.as_deref() == Some("shelffloat") {
            crate::screens::tools::shelf::floating_contents(self, ui, &ctx);
        } else if settings_inline {
            crate::screens::settings::show(self, ui, &ctx);
        } else {
            self.panel(ui, &ctx);
            ui::resize_grip(ui, "main-grip");
            if self.settings_open {
                let vid = egui::ViewportId::from_hash_of("valler-settings");
                let builder = egui::ViewportBuilder::default()
                    .with_title("Valler Settings")
                    .with_app_id("valler")
                    .with_inner_size([980.0, 680.0])
                    .with_min_inner_size([720.0, 480.0])
                    .with_decorations(false)
                    .with_transparent(true);
                ctx.show_viewport_immediate(vid, builder, |ui, _class| {
                    let c = ui.ctx().clone();
                    ui::window_surface(ui);
                    crate::screens::settings::show(self, ui, &c);
                    ui::resize_grip(ui, "settings-grip");
                    if c.input(|i| i.viewport().close_requested()) {
                        self.settings_open = false;
                    }
                });
            }
        }

        if let Some(t) = self.dirty_since {
            if t.elapsed() > Duration::from_millis(800) && !self.demo {
                self.store.save();
                self.dirty_since = None;
            } else {
                ctx.request_repaint_after(Duration::from_millis(900));
            }
        }
        crate::screens::tools::shelf::main_window_drops(self, &ctx);
        self.handle_screenshot(&ctx);
    }

    fn on_exit(&mut self) {
        if !self.demo {
            self.store.save();
        }
    }

    /// Fully transparent: the rounded surface painted in `ui` is the whole window.
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }
}
