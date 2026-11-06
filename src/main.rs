//! Valler — a cross-platform toolbox of everyday desktop utilities: system
//! monitor, keep awake, volume mixer, clipboard history, snippets, Shelf,
//! command bar, package manager and more, behind one window and tray icon.
//!
//! Layout:
//! - `core`     shared data: saved settings and the feature catalog
//! - `services` the logic of each feature, with no drawing code
//! - `ui`       visual building blocks: theme, widgets, window chrome, icon
//! - `screens`  what the person sees: panel tabs, tools and Settings pages
//! - `app`      the application state and how it runs
//! - `cli`      command-line options

#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod app;
mod cli;
mod core;
mod screens;
mod services;
mod ui;

#[cfg(test)]
mod smoke_tests;
#[cfg(test)]
mod test_util;

pub use cli::LaunchOptions;

fn main() -> eframe::Result {
    let opts = match cli::parse(std::env::args().skip(1)) {
        cli::Command::Run(o) => o,
        cli::Command::Help => {
            println!("Valler {}\n\n{}", env!("CARGO_PKG_VERSION"), cli::HELP);
            return Ok(());
        }
        cli::Command::Version => {
            println!("Valler {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        cli::Command::ExportIcon { path, size } => {
            let rgba = ui::icon_rgba(size, None);
            if let Err(e) = image::save_buffer(&path, &rgba, size, size, image::ExtendedColorType::Rgba8) {
                eprintln!("Could not write {}: {e}", path.display());
                std::process::exit(1);
            }
            return Ok(());
        }
    };
    let viewport = egui::ViewportBuilder::default()
        .with_title("Valler")
        .with_app_id("valler")
        .with_inner_size(cli::window_size(&opts))
        .with_min_inner_size([360.0, 420.0])
        // No system title bar or border on any platform: Valler draws its own
        // rounded surface, drag area, window buttons and resize grip.
        .with_decorations(false)
        .with_transparent(true)
        .with_icon(ui::app_icon());
    #[allow(unused_mut)]
    let mut native_options = eframe::NativeOptions { viewport, ..Default::default() };
    // Linux Wayland can't accept dropped files through winit; the Shelf offers
    // an opt-in XWayland mode that can.
    #[cfg(target_os = "linux")]
    if opts.screenshot.is_none() && core::store::Store::load(false).linux_x11 {
        native_options.event_loop_builder = Some(Box::new(|b| {
            use winit::platform::x11::EventLoopBuilderExtX11;
            b.with_x11();
        }));
    }
    eframe::run_native(
        "Valler",
        native_options,
        Box::new(move |cc| {
            ui::fonts::install(&cc.egui_ctx);
            ui::install_style(&cc.egui_ctx);
            Ok(Box::new(app::ValleApp::new(cc, opts)))
        }),
    )
}
