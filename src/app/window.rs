//! Window and tray behaviour: quitting, closing to the tray, showing and
//! hiding, tray menu commands and the tray readout.

use crate::core::features::Tab;
use crate::ui;
use super::*;

impl ValleApp {
    pub fn quit(&mut self, ctx: &egui::Context) {
        self.quitting = true;
        self.store.save();
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }

    /// The window's close button: tuck Valler into the tray when there is one.
    pub fn close_window(&mut self, ctx: &egui::Context) {
        if self.tray.is_some() && self.store.close_to_tray {
            self.hide_window(ctx);
        } else {
            self.quit(ctx);
        }
    }

    pub fn hide_window(&mut self, ctx: &egui::Context) {
        self.store.save();
        // Wayland compositors don't let apps hide their own windows; minimize instead.
        if cfg!(target_os = "linux") && std::env::var_os("WAYLAND_DISPLAY").is_some() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        } else {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }
        self.window_hidden = true;
    }

    pub fn show_window(&mut self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        self.window_hidden = false;
    }

    /// Handles tray clicks and menu items, and refreshes the tray readout.
    pub(super) fn tray_tick(&mut self, ctx: &egui::Context) {
        let Some(tray) = self.tray.as_ref() else { return };
        let cmds: Vec<crate::services::tray::TrayCmd> = tray.rx.try_iter().collect();
        for c in cmds {
            use crate::services::tray::TrayCmd::*;
            match c {
                Toggle => {
                    let minimized = ctx.input(|i| i.viewport().minimized.unwrap_or(false));
                    if self.window_hidden || minimized {
                        self.show_window(ctx);
                    } else {
                        self.hide_window(ctx);
                    }
                }
                Show => self.show_window(ctx),
                KeepAwake => {
                    if self.awake.active() {
                        self.awake.stop();
                    } else {
                        self.awake.start(self.store.keep_awake_minutes, self.store.keep_display_on);
                    }
                }
                CommandBar => {
                    self.tab = Tab::Utilities;
                    self.sub = Some(Sub::CommandBar);
                    self.cmd_focus = true;
                    self.show_window(ctx);
                }
                System => {
                    self.tab = Tab::System;
                    self.sub = None;
                    self.show_window(ctx);
                }
                Settings => {
                    self.settings_open = true;
                    self.show_window(ctx);
                }
                Shelf => {
                    if self.store.is_installed("shelf") {
                        self.shelf_floating = !self.shelf_floating;
                        ctx.request_repaint();
                    }
                }
                Quit => self.quit(ctx),
            }
        }
        // Closing the window from the system (Alt+F4, Dock, taskbar) also goes to the tray.
        if !self.quitting && self.store.close_to_tray && ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.hide_window(ctx);
        }
        let awake = self.awake.active();
        let s = &self.snap;
        use crate::services::tray::Readout;
        let level = |f: f32| if f > 0.85 { ui::RED } else if f > 0.6 { ui::ORANGE } else { ui::GREEN };
        let (text, gauge) = if !self.store.is_installed("tray_readouts") || !s.ready {
            (None, None)
        } else {
            match self.store.tray_readout {
                Readout::Off => (None, None),
                Readout::Cpu => (Some(format!("CPU {:.0}%", s.cpu_total)), Some((s.cpu_total / 100.0, level(s.cpu_total / 100.0)))),
                Readout::Memory => (Some(format!("Mem {:.0}%", s.mem_frac() * 100.0)), Some((s.mem_frac(), level(s.mem_frac())))),
                Readout::Battery => match &s.battery {
                    Some(b) => (Some(format!("{:.0}%", b.percent)), Some((b.percent / 100.0, if b.percent < 20.0 { ui::RED } else { ui::GREEN }))),
                    None => (None, None),
                },
                Readout::Network => (Some(format!("↓{} ↑{}", ui::fmt_rate(s.rx_rate), ui::fmt_rate(s.tx_rate))), None),
            }
        };
        if let Some(tray) = self.tray.as_mut() {
            tray.set_keep_awake(awake);
            tray.set_readout(text, gauge);
        }
    }
}
