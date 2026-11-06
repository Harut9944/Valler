//! Tray / menu bar icon on every platform: the notification area on Windows,
//! the menu bar on macOS and the StatusNotifier tray on Linux (KDE out of the
//! box; GNOME with the AppIndicator extension that most distributions ship).
//! Left click shows or hides Valler; the menu has the common actions.

use std::sync::mpsc::{Receiver, channel};
use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TrayCmd {
    Toggle,
    Show,
    KeepAwake,
    CommandBar,
    System,
    Settings,
    Quit,
}

/// What the tray shows next to (and inside) the icon.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize, Default)]
pub enum Readout {
    #[default]
    Off,
    Cpu,
    Memory,
    Battery,
    Network,
}

impl Readout {
    pub const ALL: [Readout; 5] = [Readout::Off, Readout::Cpu, Readout::Memory, Readout::Battery, Readout::Network];
    pub fn label(self) -> &'static str {
        match self {
            Readout::Off => "Icon only",
            Readout::Cpu => "Processor usage",
            Readout::Memory => "Memory usage",
            Readout::Battery => "Battery charge",
            Readout::Network => "Network speed",
        }
    }
}

pub struct Tray {
    icon: TrayIcon,
    awake: CheckMenuItem,
    pub rx: Receiver<TrayCmd>,
    last_icon: Option<u8>,
    last_text: String,
}

const ICON_PX: u32 = 32;

fn make_icon(gauge: Option<(f32, egui::Color32)>) -> Option<Icon> {
    Icon::from_rgba(crate::ui::icon_rgba(ICON_PX, gauge), ICON_PX, ICON_PX).ok()
}

impl Tray {
    pub fn new(ctx: &egui::Context) -> Option<Self> {
        let menu = Menu::new();
        let show = MenuItem::new("Show Valler", true, None);
        let awake = CheckMenuItem::new("Keep awake", true, false, None);
        let cmd = MenuItem::new("Command bar…", true, None);
        let system = MenuItem::new("System monitor", true, None);
        let settings = MenuItem::new("Settings…", true, None);
        let quit = MenuItem::new("Quit Valler", true, None);
        menu.append_items(&[
            &show,
            &PredefinedMenuItem::separator(),
            &awake,
            &cmd,
            &system,
            &PredefinedMenuItem::separator(),
            &settings,
            &quit,
        ])
        .ok()?;

        let (tx, rx) = channel();
        let ids = [
            (show.id().clone(), TrayCmd::Show),
            (awake.id().clone(), TrayCmd::KeepAwake),
            (cmd.id().clone(), TrayCmd::CommandBar),
            (system.id().clone(), TrayCmd::System),
            (settings.id().clone(), TrayCmd::Settings),
            (quit.id().clone(), TrayCmd::Quit),
        ];
        let (tx2, ctx2) = (tx.clone(), ctx.clone());
        // Handlers run even while the window is hidden; they wake the app up.
        MenuEvent::set_event_handler(Some(move |e: MenuEvent| {
            if let Some((_, c)) = ids.iter().find(|(id, _)| *id == e.id) {
                let _ = tx2.send(*c);
                ctx2.request_repaint();
            }
        }));
        let ctx3 = ctx.clone();
        TrayIconEvent::set_event_handler(Some(move |e: TrayIconEvent| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state, .. } = e {
                // Linux reports activation without a release state; others send down+up.
                if button_state == MouseButtonState::Up || cfg!(target_os = "linux") {
                    let _ = tx.send(TrayCmd::Toggle);
                    ctx3.request_repaint();
                }
            }
        }));

        let icon = TrayIconBuilder::new()
            .with_id("valler")
            .with_tooltip("Valler")
            .with_icon(make_icon(None)?)
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(false)
            .build()
            .ok()?;
        Some(Self { icon, awake, rx, last_icon: None, last_text: String::new() })
    }

    pub fn set_keep_awake(&self, on: bool) {
        if self.awake.is_checked() != on {
            self.awake.set_checked(on);
        }
    }

    /// Updates the tooltip, the text beside the icon (macOS menu bar) and the
    /// gauge drawn inside the icon. Only touches the system when something changed.
    pub fn set_readout(&mut self, text: Option<String>, gauge: Option<(f32, egui::Color32)>) {
        let t = text.clone().unwrap_or_default();
        if t != self.last_text {
            let tip = if t.is_empty() { "Valler".to_string() } else { format!("Valler · {t}") };
            let _ = self.icon.set_tooltip(Some(tip));
            if cfg!(target_os = "macos") {
                self.icon.set_title(text.as_deref());
            }
            self.last_text = t;
        }
        // Redraw the icon only when the gauge moves by a visible step.
        let step = gauge.map(|(f, _)| (f.clamp(0.0, 1.0) * 16.0).round() as u8 + 1);
        if step != self.last_icon {
            self.last_icon = step;
            let _ = self.icon.set_icon(make_icon(gauge));
        }
    }
}

#[cfg(test)]
mod tests {
    /// Needs a desktop session with a tray; run with `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn registers_with_the_desktop_tray() {
        let ctx = egui::Context::default();
        let mut t = super::Tray::new(&ctx).expect("tray should be created");
        t.set_readout(Some("CPU 42%".into()), Some((0.42, egui::Color32::GREEN)));
        t.set_keep_awake(true);
        std::thread::sleep(std::time::Duration::from_millis(1500));
    }
}
