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
| Clipboard history, auto clear, clean URL (manual + automatic) | ✓ | ✓ | ✓ |
| Snippets (copy with variables), Scratchpad | ✓ | ✓ | ✓ |
| Quit apps, Port manager, Cleaner (moves to trash) | ✓ | ✓ | ✓ |
| Alerts: CPU load/temperature, memory, low disk, low battery (desktop notifications) | ✓ | ✓ | ✓ |
| Command bar (Ctrl/Cmd+K): tools, clipboard, snippets, notes, calculator, unit conversion, web search | ✓ | ✓ | ✓ |
| Quick toggles: dark mode, lock screen, display off, screen saver, empty trash | ✓ | ✓ (GNOME, KDE) | ✓ |
| Cleaning mode (full-screen cover, press Esc several times to finish) | ✓ | ✓ | ✓ |
| Command bar app search and launch | ✓ (Start Menu) | ✓ (.desktop, Flatpak) | ✓ (/Applications) |
| Timer with notification | ✓ | ✓ | ✓ |
| Open at login | ✓ | ✓ (XDG autostart) | ✓ |
| Package manager, App updates, Uninstaller | ✓ (winget) | ✓ (Flatpak, Homebrew) | ✓ (Homebrew) |
| Media tools: convert, resize and compress images | ✓ | ✓ | ✓ |
| Paste as plain text | ✓ | ✓ | ✓ |
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

## Options

```
valler [--tab <name>] [--settings] [--screenshot <file.png>]
```

`--tab` opens on keepawake, mixer, system, network, disk, power, utilities or
controls. `--screenshot` draws about 120 frames, saves the window as PNG and exits
(it uses sample data and never touches your saved settings).

Settings are stored in `~/.config/Valler`, `%APPDATA%\Valler` or
`~/Library/Application Support/Valler`.
