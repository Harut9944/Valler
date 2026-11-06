//! The feature catalog: every tool Valler knows about, which group it belongs
//! to, and how well it works on the system it is running on.

use std::collections::BTreeSet;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Group {
    WindowsDock,
    MouseKeyboard,
    ClipboardFiles,
    Sound,
    EnergyDisplay,
    Tools,
    Monitor,
}

impl Group {
    pub const ALL: [Group; 7] = [
        Group::Monitor,
        Group::Sound,
        Group::EnergyDisplay,
        Group::ClipboardFiles,
        Group::Tools,
        Group::WindowsDock,
        Group::MouseKeyboard,
    ];
    pub fn title(self) -> &'static str {
        match self {
            Group::WindowsDock => "Windows and taskbar",
            Group::MouseKeyboard => "Mouse and keyboard",
            Group::ClipboardFiles => "Clipboard and files",
            Group::Sound => "Sound",
            Group::EnergyDisplay => "Energy and display",
            Group::Tools => "Tools",
            Group::Monitor => "System monitor",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Support {
    /// Works on Windows, Linux and macOS.
    Everywhere,
    /// Works here, with a limitation explained in the note.
    Partial(&'static str),
    /// Works only on Linux for now (PulseAudio / PipeWire).
    LinuxOnly,
    /// Planned; not part of this version.
    Later,
}

impl Support {
    pub fn works_here(self) -> bool {
        match self {
            Support::Everywhere | Support::Partial(_) => true,
            Support::LinuxOnly => cfg!(target_os = "linux"),
            Support::Later => false,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Support::Everywhere => "Works everywhere",
            Support::Partial(_) => "Works, with limits",
            Support::LinuxOnly if cfg!(target_os = "linux") => "Works on Linux",
            Support::LinuxOnly => "Linux only for now",
            Support::Later => "Coming later",
        }
    }
}

pub struct Feature {
    pub id: &'static str,
    pub group: Group,
    pub name: &'static str,
    pub desc: &'static str,
    pub icon: &'static str,
    pub support: Support,
    /// What it costs while idle, like the original's energy labels.
    pub energy: &'static str,
    /// Has an on/off switch in the Controls tab.
    pub switchable: bool,
}

use egui_phosphor::regular as ph;

pub const FEATURES: &[Feature] = &[
    // System monitor
    Feature { id: "monitor_cpu", group: Group::Monitor, name: "Processor", desc: "CPU usage per core, load and history graph", icon: ph::CPU, support: Support::Everywhere, energy: "Checks every second", switchable: false },
    Feature { id: "monitor_memory", group: Group::Monitor, name: "Memory", desc: "Used memory, swap and pressure", icon: ph::MEMORY, support: Support::Everywhere, energy: "Checks every second", switchable: false },
    Feature { id: "monitor_temps", group: Group::Monitor, name: "Temperatures", desc: "CPU, GPU, drive and battery sensors", icon: ph::THERMOMETER, support: Support::Partial("Windows only shows sensors the system exposes; some PCs report none"), energy: "Checks every second", switchable: false },
    Feature { id: "monitor_gpu", group: Group::Monitor, name: "Graphics usage", desc: "GPU load and video memory", icon: ph::GRAPHICS_CARD, support: Support::Later, energy: "Checks every second", switchable: false },
    Feature { id: "monitor_network", group: Group::Monitor, name: "Network", desc: "Live traffic, totals, local IP and a speed test", icon: ph::GLOBE, support: Support::Everywhere, energy: "Checks every second", switchable: false },
    Feature { id: "monitor_disk", group: Group::Monitor, name: "Disks", desc: "Free space per drive and read/write activity", icon: ph::HARD_DRIVES, support: Support::Everywhere, energy: "Checks every second", switchable: false },
    Feature { id: "monitor_power", group: Group::Monitor, name: "Battery and power", desc: "Charge, health, cycles, time left and power draw", icon: ph::LIGHTNING, support: Support::Everywhere, energy: "Checks every second", switchable: false },
    Feature { id: "monitor_alerts", group: Group::Monitor, name: "Alerts", desc: "Notifications for sustained processor load, high temperatures, memory, low disk space or low battery", icon: ph::BELL_RINGING, support: Support::Everywhere, energy: "Nothing at rest", switchable: false },
    Feature { id: "fan_control", group: Group::Monitor, name: "Fan control", desc: "Manual fan speeds and temperature curves", icon: ph::FAN, support: Support::Later, energy: "Nothing at rest", switchable: false },
    // Sound
    Feature { id: "mixer", group: Group::Sound, name: "Volume mixer", desc: "Set volume per app, boost quiet audio past 100% and pick each app's output", icon: ph::SLIDERS_HORIZONTAL, support: Support::LinuxOnly, energy: "Nothing at rest", switchable: false },
    Feature { id: "mic_mute", group: Group::Sound, name: "Microphone mute", desc: "Mute or unmute the default microphone with one click", icon: ph::MICROPHONE_SLASH, support: Support::LinuxOnly, energy: "Nothing at rest", switchable: false },
    Feature { id: "music_block", group: Group::Sound, name: "Music app blocker", desc: "Stop unwanted music app launches from media keys", icon: ph::MUSIC_NOTES, support: Support::Later, energy: "Listens to the keyboard", switchable: true },
    // Energy and display
    Feature { id: "keep_awake", group: Group::EnergyDisplay, name: "Keep awake", desc: "Keep the computer and display awake for a while or until you stop it", icon: ph::MOON_STARS, support: Support::Everywhere, energy: "Nothing at rest", switchable: false },
    Feature { id: "brightness", group: Group::EnergyDisplay, name: "Displays", desc: "Brightness for every monitor, with hardware control where supported", icon: ph::SUN, support: Support::Later, energy: "Nothing at rest", switchable: false },
    Feature { id: "bluetooth_sleep", group: Group::EnergyDisplay, name: "Bluetooth on sleep", desc: "Turn Bluetooth off during sleep and back on at wake", icon: ph::BLUETOOTH, support: Support::Later, energy: "Nothing at rest", switchable: true },
    // Clipboard and files
    Feature { id: "clipboard", group: Group::ClipboardFiles, name: "Clipboard history", desc: "Keeps copied text so you can reuse it later. Everything stays on this computer", icon: ph::CLIPBOARD_TEXT, support: Support::Partial("Text only for now; images and files come later"), energy: "Checks twice a second", switchable: true },
    Feature { id: "clip_autoclear", group: Group::ClipboardFiles, name: "Auto clear clipboard", desc: "Empties the clipboard a while after you copy, keeping saved history", icon: ph::BROOM, support: Support::Everywhere, energy: "Nothing at rest", switchable: true },
    Feature { id: "url_clean", group: Group::ClipboardFiles, name: "Clean URL", desc: "Removes tracking parameters from copied links, by hand or automatically", icon: ph::LINK, support: Support::Everywhere, energy: "Nothing at rest", switchable: true },
    Feature { id: "paste_plain", group: Group::ClipboardFiles, name: "Paste as plain text", desc: "Strips formatting from what you copied so it pastes as plain text", icon: ph::TEXT_T, support: Support::Everywhere, energy: "Nothing at rest", switchable: false },
    Feature { id: "snippets", group: Group::ClipboardFiles, name: "Text snippets", desc: "Saved text with date, time and clipboard variables, ready to copy", icon: ph::TEXT_AA, support: Support::Partial("Typing a trigger to expand it comes later; copy from the library for now"), energy: "Nothing at rest", switchable: false },
    Feature { id: "shelf", group: Group::ClipboardFiles, name: "Shelf", desc: "Park files, text and links while dragging", icon: ph::TRAY, support: Support::Later, energy: "Listens to the mouse", switchable: true },
    // Tools
    Feature { id: "scratchpad", group: Group::Tools, name: "Scratchpad", desc: "Autosaved notes in tabs, with export to a text file", icon: ph::NOTE_PENCIL, support: Support::Everywhere, energy: "Nothing at rest", switchable: false },
    Feature { id: "quit_apps", group: Group::Tools, name: "Quit apps", desc: "See running programs with their CPU and memory, and quit or force quit them", icon: ph::X_SQUARE, support: Support::Everywhere, energy: "Nothing at rest", switchable: false },
    Feature { id: "ports", group: Group::Tools, name: "Port manager", desc: "Find programs listening on network ports and stop them", icon: ph::PLUGS_CONNECTED, support: Support::Everywhere, energy: "Nothing at rest", switchable: false },
    Feature { id: "cleaner", group: Group::Tools, name: "Cleaner", desc: "Find caches, logs and temporary files and move them to the trash", icon: ph::SPARKLE, support: Support::Everywhere, energy: "Nothing at rest", switchable: false },
    Feature { id: "command_bar", group: Group::Tools, name: "Command bar", desc: "Search tools, clipboard, snippets and notes from one field. Calculate, convert units or search the web", icon: ph::MAGNIFYING_GLASS, support: Support::Partial("Searches inside Valler; searching installed apps and files comes later. Press Ctrl+K in the window"), energy: "Nothing at rest", switchable: false },
    Feature { id: "quick_toggles", group: Group::Tools, name: "Quick toggles", desc: "Switch dark mode, lock the screen, turn the display off, start the screen saver or empty the trash", icon: ph::TOGGLE_LEFT, support: Support::Everywhere, energy: "Nothing at rest", switchable: false },
    Feature { id: "timer", group: Group::Tools, name: "Timer", desc: "A countdown with a notification when it ends, for focus sessions, tea or breaks", icon: ph::TIMER, support: Support::Everywhere, energy: "Nothing at rest", switchable: false },
    Feature { id: "cleaning_mode", group: Group::Tools, name: "Cleaning mode", desc: "Covers the screen and swallows typing while you wipe the keyboard and display", icon: ph::KEYBOARD, support: Support::Partial("System shortcuts such as Alt+Tab or the Windows key still reach the system"), energy: "Nothing at rest", switchable: false },
    Feature { id: "screenshot", group: Group::Tools, name: "Screen capture", desc: "Screenshots, recordings, text recognition and color picking", icon: ph::CAMERA, support: Support::Later, energy: "Nothing at rest", switchable: false },
    Feature { id: "uninstaller", group: Group::Tools, name: "Uninstaller", desc: "Remove apps you installed from Flatpak, Homebrew or winget", icon: ph::TRASH, support: Support::Partial("Apps installed with the system package manager (apt, dnf, pacman) or by hand are not listed yet"), energy: "Nothing at rest", switchable: false },
    Feature { id: "packages", group: Group::Tools, name: "Package manager", desc: "Search, install, update and remove apps without a terminal", icon: ph::PACKAGE, support: Support::Partial("Uses Flatpak or Homebrew on Linux, Homebrew on macOS and winget on Windows"), energy: "Nothing at rest", switchable: false },
    Feature { id: "app_updates", group: Group::Tools, name: "App updates", desc: "See which apps have updates and install them together", icon: ph::ARROW_CIRCLE_UP, support: Support::Partial("Covers apps from Flatpak, Homebrew and winget"), energy: "Nothing at rest", switchable: false },
    Feature { id: "media", group: Group::Tools, name: "Media tools", desc: "Convert, resize and compress images locally", icon: ph::IMAGES, support: Support::Partial("Images only for now; video tools come later"), energy: "Nothing at rest", switchable: false },
    Feature { id: "island", group: Group::Tools, name: "Activity island", desc: "Music, timers, calendar and downloads in a floating island at the top", icon: ph::PILL, support: Support::Later, energy: "Nothing at rest", switchable: true },
    // Windows and taskbar
    Feature { id: "switcher", group: Group::WindowsDock, name: "App switcher", desc: "Switch between apps and windows with previews and search", icon: ph::STACK, support: Support::Later, energy: "Listens to the keyboard", switchable: true },
    Feature { id: "window_layout", group: Group::WindowsDock, name: "Window layout", desc: "Snap windows into halves, thirds and corners, or move them between displays", icon: ph::LAYOUT, support: Support::Later, energy: "Nothing at rest", switchable: true },
    Feature { id: "taskbar_preview", group: Group::WindowsDock, name: "Taskbar preview", desc: "Window previews when hovering the taskbar or dock", icon: ph::APP_WINDOW, support: Support::Later, energy: "Listens to the mouse", switchable: true },
    Feature { id: "quit_protect", group: Group::WindowsDock, name: "Quit protection", desc: "Prevent accidental quit or close shortcuts with a hold or double press", icon: ph::SHIELD_CHECK, support: Support::Later, energy: "Listens to the keyboard", switchable: true },
    // Mouse and keyboard
    Feature { id: "smooth_scroll", group: Group::MouseKeyboard, name: "Smooth scrolling", desc: "Give your mouse wheel a fluid glide", icon: ph::MOUSE_SCROLL, support: Support::Later, energy: "Listens to the mouse", switchable: true },
    Feature { id: "scroll_invert", group: Group::MouseKeyboard, name: "Scroll direction", desc: "Invert mouse scrolling independently of the touchpad", icon: ph::ARROWS_DOWN_UP, support: Support::Later, energy: "Listens to the mouse", switchable: true },
    Feature { id: "focus_follows", group: Group::MouseKeyboard, name: "Focus follows mouse", desc: "Bring the window under the pointer forward after a pause", icon: ph::CURSOR, support: Support::Later, energy: "Listens to the mouse", switchable: true },
    Feature { id: "click_debounce", group: Group::MouseKeyboard, name: "Extra click filter", desc: "Ignore accidental double clicks from worn mouse buttons", icon: ph::MOUSE, support: Support::Later, energy: "Listens to the mouse", switchable: true },
    Feature { id: "key_debounce", group: Group::MouseKeyboard, name: "Key debounce", desc: "Filter repeated letters caused by a worn keyboard", icon: ph::KEYBOARD, support: Support::Later, energy: "Listens to the keyboard", switchable: true },
    Feature { id: "super_key", group: Group::MouseKeyboard, name: "Super key", desc: "Use Caps Lock as a shortcut combination", icon: ph::COMMAND, support: Support::Later, energy: "Listens to the keyboard", switchable: true },
];

pub fn get(id: &str) -> Option<&'static Feature> {
    FEATURES.iter().find(|f| f.id == id)
}

pub fn default_installed() -> BTreeSet<String> {
    FEATURES
        .iter()
        .filter(|f| f.support.works_here())
        .map(|f| f.id.to_string())
        .collect()
}

pub fn default_enabled() -> BTreeSet<String> {
    ["clipboard", "url_clean"].iter().map(|s| s.to_string()).collect()
}

pub struct Preset {
    pub name: &'static str,
    pub desc: &'static str,
    pub icon: &'static str,
    pub features: &'static [&'static str],
}

pub const PRESETS: &[Preset] = &[
    Preset {
        name: "Essentials",
        desc: "Volume mixer, system monitor and keep awake.",
        icon: ph::STAR,
        features: &["mixer", "mic_mute", "keep_awake", "monitor_cpu", "monitor_memory", "monitor_temps", "monitor_network", "monitor_disk", "monitor_power", "monitor_alerts"],
    },
    Preset {
        name: "Everyday tools",
        desc: "Clipboard history, snippets, scratchpad and link cleaning.",
        icon: ph::TOOLBOX,
        features: &["clipboard", "clip_autoclear", "url_clean", "snippets", "scratchpad", "quit_apps", "ports", "cleaner", "command_bar", "quick_toggles", "cleaning_mode", "timer", "paste_plain", "media", "packages", "app_updates", "uninstaller"],
    },
    Preset {
        name: "Battery and quiet",
        desc: "A lean monitor with battery, memory and processor. Nothing listens to input.",
        icon: ph::BATTERY_HIGH,
        features: &["monitor_cpu", "monitor_memory", "monitor_power"],
    },
];

/// Panel tabs, in display order.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tab {
    KeepAwake,
    Mixer,
    System,
    Network,
    Disk,
    Power,
    Utilities,
    Controls,
}

impl Tab {
    pub const ALL: [Tab; 8] = [
        Tab::KeepAwake,
        Tab::Mixer,
        Tab::System,
        Tab::Network,
        Tab::Disk,
        Tab::Power,
        Tab::Utilities,
        Tab::Controls,
    ];
    pub fn id(self) -> &'static str {
        match self {
            Tab::KeepAwake => "keepawake",
            Tab::Mixer => "mixer",
            Tab::System => "system",
            Tab::Network => "network",
            Tab::Disk => "disk",
            Tab::Power => "power",
            Tab::Utilities => "utilities",
            Tab::Controls => "controls",
        }
    }
    pub fn from_id(s: &str) -> Option<Tab> {
        Tab::ALL.into_iter().find(|t| t.id() == s)
    }
    pub fn title(self) -> &'static str {
        match self {
            Tab::KeepAwake => "Keep awake",
            Tab::Mixer => "Volume mixer",
            Tab::System => "System",
            Tab::Network => "Network",
            Tab::Disk => "Disks",
            Tab::Power => "Power",
            Tab::Utilities => "Utilities",
            Tab::Controls => "Controls",
        }
    }
    pub fn icon(self) -> &'static str {
        match self {
            Tab::KeepAwake => ph::MOON_STARS,
            Tab::Mixer => ph::SLIDERS_HORIZONTAL,
            Tab::System => ph::CPU,
            Tab::Network => ph::GLOBE,
            Tab::Disk => ph::HARD_DRIVES,
            Tab::Power => ph::LIGHTNING,
            Tab::Utilities => ph::WRENCH,
            Tab::Controls => ph::TOGGLE_RIGHT,
        }
    }
    /// Features that make this tab appear.
    pub fn features(self) -> &'static [&'static str] {
        match self {
            Tab::KeepAwake => &["keep_awake"],
            Tab::Mixer => &["mixer", "mic_mute"],
            Tab::System => &["monitor_cpu", "monitor_memory", "monitor_temps"],
            Tab::Network => &["monitor_network"],
            Tab::Disk => &["monitor_disk"],
            Tab::Power => &["monitor_power"],
            Tab::Utilities => &["command_bar", "quick_toggles", "clipboard", "snippets", "scratchpad", "url_clean", "quit_apps", "ports", "cleaner", "timer", "cleaning_mode", "packages", "app_updates", "uninstaller", "media"],
            Tab::Controls => &["clipboard", "clip_autoclear", "url_clean"],
        }
    }
}
