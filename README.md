# Valler

Everyday desktop utilities in one window, for Windows, Linux (GNOME and KDE) and
macOS (Apple silicon and Intel). A cross-platform Rust rewrite of an open-source
macOS menu-bar utility collection (GPL-3.0-or-later).

## What works in this version

| Feature | Windows | Linux | macOS |
|---|---|---|---|
| System monitor: CPU, per-core, GPU, memory, temperatures, uptime | ✓ | ✓ | ✓ |
| Network: live traffic, totals, local IP, interfaces, speed test | ✓ | ✓ | ✓ |
| Disks: free space, read/write activity | ✓ | ✓ | ✓ |
| Battery: charge, health, cycles, time left, power draw | ✓ | ✓ | ✓ |
| Keep awake (timer, keep display on) | ✓ | ✓ | ✓ |
| Volume mixer: per-app volume and mute | ✓ (WASAPI sessions, up to 100 %) | ✓ (PulseAudio / PipeWire, boost to 200 %) | apps playing are listed; per-app volume next |
| Output and microphone: volume, mute, default device | ✓ (device switch opens Sound settings) | ✓ | ✓ (CoreAudio) |
| Per-app output device | — | ✓ | — |
| Clipboard history for text, images and files; auto clear; clean URL | ✓ | ✓ | ✓ |
| Snippets (copy with variables), Scratchpad | ✓ | ✓ | ✓ |
| Quit apps, Port manager, Cleaner (moves to trash) | ✓ | ✓ | ✓ |
| Alerts: CPU load/temperature, memory, low disk, low battery (desktop notifications) | ✓ | ✓ | ✓ |
| Command bar (Ctrl/Cmd+K): tools, clipboard, snippets, notes, calculator, unit conversion, web search | ✓ | ✓ | ✓ |
| Quick toggles: dark mode, lock screen, display off, screen saver, empty trash | ✓ | ✓ (GNOME, KDE) | ✓ |
| Cleaning mode (full-screen cover, press Esc several times to finish) | ✓ | ✓ | ✓ |
| Command bar app search and launch | ✓ (Start Menu) | ✓ (.desktop, Flatpak) | ✓ (/Applications) |
| Timer with notification | ✓ | ✓ | ✓ |
| Keep awake automatically (while apps run / while plugged in, Any or All) | ✓ | ✓ | ✓ |
| Command bar: colours (HEX/RGB/HSL), date maths, emoji | ✓ | ✓ | ✓ |
| Scratchpad Markdown preview | ✓ | ✓ | ✓ |
| Displays: brightness per screen | ✓ (laptop screens) | ✓ (backlight; external monitors via ddcutil) | — |
| Software dimming (click-through overlay, also below minimum brightness) | ✓ | ✓ | ✓ |
| Open at login | ✓ | ✓ (XDG autostart) | ✓ |
| Package manager, App updates, Uninstaller | ✓ (winget) | ✓ (Flatpak, Homebrew) | ✓ (Homebrew) |
| Media tools: convert, resize and compress images | ✓ | ✓ | ✓ |
| Paste as plain text | ✓ | ✓ | ✓ |
| Shelf: park files, folders, images, text and links; copy, move, open; floating on-top window | ✓ (drop files) | ✓ (drop on X11; paste or opt-in XWayland on Wayland) | ✓ (drop files) |
| Tray / menu bar icon: click to show or hide, menu with quick actions, close to tray | ✓ (notification area) | ✓ (KDE; GNOME with the AppIndicator extension) | ✓ (menu bar) |
| Tray readouts: CPU, memory, battery or network on the icon | ✓ (gauge + tooltip) | ✓ (gauge + tooltip) | ✓ (gauge + text) |
| Taskbar / launcher icon | ✓ | ✓ (installs a launcher entry for Wayland) | ✓ (Dock) |
| Settings with Features hub, bundles, platform support table | ✓ | ✓ | ✓ |

Valler draws its own frameless, rounded window on every platform: drag it by the
header, use the round buttons to minimize or close, and the corner grip to resize.

Window management, mouse tweaks, screen capture and similar features are listed
in the Features hub as *Coming later*.

## Build

Install Rust (https://rustup.rs), then:

```sh
cargo build --release
./target/release/valler
```

Linux build needs the usual windowing headers (`libxkbcommon`, `wayland`,
`libx11`/`libxcb`). The mixer uses the `pactl` tool at run time
(package `pulseaudio-utils` or `libpulse`).

For an Intel Mac from an Apple silicon Mac: `rustup target add x86_64-apple-darwin`
and `cargo build --release --target x86_64-apple-darwin`.

## Tests

```sh
cargo test               # 133 tests: every feature's logic plus every screen drawn off-screen
cargo test -- --ignored  # 4 more that need a desktop session (tray icon, keep awake, icon previews, real screens)
```

Each feature has tests next to its code (link cleaning, snippets, calculator and
units, alerts, monitor, keep awake, mixer parsing, clipboard-free Shelf
operations, media conversion, cleaner, app search, packages, ports, speed test,
tray, settings storage and the feature catalog). The interface tests in
`src/smoke_tests.rs` build Valler on sample data and draw every panel tab,
Utilities tool, Settings page and the floating Shelf.

## Project layout

```
src/
  main.rs        start-up: options, window, run
  cli.rs         command-line options (pure, tested)
  app/           the application state and how it runs
    mod.rs         ValleApp: state and construction
    nav.rs         open tool / Settings page, --tab ids
    panel.rs       header, tab bar, content, footer
    window.rs      quit, close to tray, show/hide, tray menu
    background.rs  mixer polling, clipboard collection
    automation.rs  keep-awake automation tick
    dimmer.rs      software dimming overlay
    lifecycle.rs   eframe entry points
    screenshot.rs  --screenshot mode
  core/          shared data: saved settings, feature catalog
  services/      feature logic, no drawing code (each tested next to its code)
    command_bar/   calc, units, colors, dates, emoji, fuzzy
    keepawake/     inhibitor + automation rules
    clipboard/     watcher, history rules, image storage
    display/       brightness: linux.rs, windows.rs behind one API
    mixer/         linux.rs, windows.rs, macos.rs behind one API
    ...            alerts, apps, cleaner, clipboard, markdown, media, monitor,
                   packages, ports, quick_toggles, shelf, snippets, speedtest,
                   tray, urlclean, autostart, desktop_entry
  ui/            visual building blocks: theme, widgets, chrome, icon,
                 format, fonts, markdown renderer
  screens/       what the person sees, one file per page
    tabs/          keep awake, mixer, system, network, disk, power, controls
    tools/         each Utilities tool (shelf/ split into page, tiles,
                   floating window and actions)
    settings/      sidebar + one file per Settings page
  smoke_tests.rs every screen drawn off-screen on sample data
```

Rules of thumb: screens never talk to the operating system directly — they
call `services`; services never draw; anything platform-specific sits behind
one function with `#[cfg]` variants (see `services/mixer`).

## Options

```
valler [--tab <name>] [--settings] [--screenshot <file.png>]
```

`--tab` opens on keepawake, mixer, system, network, disk, power, utilities or
controls. `--screenshot` draws about 120 frames, saves the window as PNG and exits
(it uses sample data and never touches your saved settings).

Settings are stored in `~/.config/Valler`, `%APPDATA%\Valler` or
`~/Library/Application Support/Valler`.
