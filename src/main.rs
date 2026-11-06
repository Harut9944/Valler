//! Valler — a cross-platform toolbox of everyday desktop utilities:
//! system monitor, keep awake, volume mixer, clipboard history, snippets,
//! scratchpad, link cleaner, quit apps, port manager and cleaner.

#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod alerts;
mod app;
mod apps;
mod autostart;
mod clipboard;
mod command_bar;
mod cleaner;
mod desktop_entry;
mod features;
mod fonts;
mod keepawake_ctl;
mod media;
mod mixer;
mod monitor;
mod packages;
mod ports;
mod quick_toggles;
mod settings_ui;
mod snippets;
mod speedtest;
mod store;
mod tabs;
mod tools_extra;
mod tools_more;
mod tools_ui;
mod tray;
mod ui;
mod urlclean;

use std::path::PathBuf;

#[derive(Default, Clone)]
pub struct LaunchOptions {
    /// Save a picture of the window after ~120 frames and exit.
    pub screenshot: Option<PathBuf>,
    /// Open on this panel tab (keepawake, mixer, system, network, disk, power, utilities, controls).
    pub tab: Option<String>,
    /// Open the Settings window right away (or render it in screenshot mode).
    pub settings: bool,
}

fn parse_args() -> LaunchOptions {
    let mut opts = LaunchOptions::default();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--screenshot" => opts.screenshot = args.next().map(PathBuf::from),
            "--tab" => opts.tab = args.next(),
            "--settings" => opts.settings = true,
            "-h" | "--help" => {
                println!(
                    "Valler {}\n\nUsage: valler [--tab <name>] [--settings] [--screenshot <file.png>]\n\n  --tab <name>          open on a panel tab: keepawake, mixer, system, network, disk, power, utilities, controls\n  --settings            open the Settings window\n  --screenshot <file>   draw ~120 frames, save the window as PNG and exit",
                    env!("CARGO_PKG_VERSION")
                );
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("Valler {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            _ => {}
        }
    }
    opts
}

fn main() -> eframe::Result {
    let opts = parse_args();
    let size = if opts.settings && opts.screenshot.is_some() {
        [1000.0, 700.0]
    } else {
        [400.0, 820.0]
    };
    let viewport = egui::ViewportBuilder::default()
        .with_title("Valler")
        .with_app_id("valler")
        .with_inner_size(size)
        .with_min_inner_size([360.0, 420.0])
        // No system title bar or border on any platform: Valler draws its own
        // rounded surface, drag area, window buttons and resize grip.
        .with_decorations(false)
        .with_transparent(true)
        .with_icon(ui::app_icon());
    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "Valler",
        native_options,
        Box::new(move |cc| {
            fonts::install(&cc.egui_ctx);
            ui::install_style(&cc.egui_ctx);
            Ok(Box::new(app::ValleApp::new(cc, opts)))
        }),
    )
}
