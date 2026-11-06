//! The logic behind each feature, free of drawing code: system readings,
//! sound, clipboard, files, packages, network and the tray. Screens call into
//! these; tests exercise them directly.

pub mod alerts;
pub mod apps;
pub mod autostart;
pub mod cleaner;
pub mod display;
pub mod clipboard;
pub mod command_bar;
pub mod desktop_entry;
pub mod keepawake;
pub mod markdown;
pub mod media;
pub mod mixer;
pub mod monitor;
pub mod packages;
pub mod ports;
pub mod quick_toggles;
pub mod shelf;
pub mod snippets;
pub mod speedtest;
pub mod tray;
pub mod urlclean;
