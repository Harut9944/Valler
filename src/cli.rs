//! Command-line options. Parsing is pure (it takes the arguments and returns
//! what to do), so it is easy to test; `main` acts on the result.

use std::path::PathBuf;

#[derive(Default, Clone, Debug, PartialEq)]
pub struct LaunchOptions {
    /// Save a picture of the window after ~120 frames and exit.
    pub screenshot: Option<PathBuf>,
    /// Open on a panel tab (keepawake, mixer, system, …) or a tool (shelf, timer, …).
    pub tab: Option<String>,
    /// Open the Settings window right away (or render it in screenshot mode).
    pub settings: bool,
}

/// What the command line asks for.
#[derive(Debug, PartialEq)]
pub enum Command {
    Run(LaunchOptions),
    Help,
    Version,
    /// Write the app icon as a PNG (used by the release scripts for the
    /// macOS .app, the Linux launcher icon and the Windows package).
    ExportIcon { path: PathBuf, size: u32 },
}

pub const HELP: &str = "Usage: valler [--tab <name>] [--settings] [--screenshot <file.png>]

  --tab <name>          open on a panel tab (keepawake, mixer, system, network, disk,
                        power, utilities, controls) or a tool (commandbar, shelf,
                        clipboard, timer, packages, media, …)
  --settings            open the Settings window (with --tab: a page such as monitor)
  --screenshot <file>   draw ~120 frames, save the window as PNG and exit
  --export-icon <file.png> [size]
                        write the app icon as a PNG (default 1024 px) and exit
  -h, --help            show this help
  -V, --version         show the version";

pub fn parse(args: impl IntoIterator<Item = String>) -> Command {
    let mut opts = LaunchOptions::default();
    let mut args = args.into_iter();
    while let Some(a) = args.next() {
        match a.as_str() {
            "--screenshot" => opts.screenshot = args.next().map(PathBuf::from),
            "--tab" => opts.tab = args.next(),
            "--settings" => opts.settings = true,
            "-h" | "--help" => return Command::Help,
            "-V" | "--version" => return Command::Version,
            "--export-icon" => {
                let Some(path) = args.next() else { return Command::Help };
                let size = args.next().and_then(|s| s.parse().ok()).filter(|s| (16..=2048).contains(s)).unwrap_or(1024);
                return Command::ExportIcon { path: PathBuf::from(path), size };
            }
            _ => {} // Unknown flags are ignored, so launchers adding their own never break Valler.
        }
    }
    Command::Run(opts)
}

/// Window size for the launch: Settings and the floating Shelf are rendered
/// in the main window in screenshot mode, so it takes their size.
pub fn window_size(opts: &LaunchOptions) -> [f32; 2] {
    match (&opts.screenshot, opts.settings, opts.tab.as_deref()) {
        (Some(_), true, _) => [1000.0, 700.0],
        (Some(_), false, Some("shelffloat")) => [372.0, 330.0],
        _ => [400.0, 820.0],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &str) -> Vec<String> {
        s.split_whitespace().map(String::from).collect()
    }

    #[test]
    fn parses_every_option() {
        let Command::Run(o) = parse(args("--tab mixer --settings --screenshot shots/a.png")) else { panic!() };
        assert_eq!(o.tab.as_deref(), Some("mixer"));
        assert!(o.settings);
        assert_eq!(o.screenshot, Some(PathBuf::from("shots/a.png")));
        assert_eq!(parse(args("")), Command::Run(LaunchOptions::default()));
    }

    #[test]
    fn help_and_version_win_and_unknown_flags_are_ignored() {
        assert_eq!(parse(args("--tab x --help")), Command::Help);
        assert_eq!(parse(args("-V")), Command::Version);
        assert_eq!(parse(args("--unknown --tab shelf")), Command::Run(LaunchOptions { tab: Some("shelf".into()), ..Default::default() }));
        // A flag missing its value is simply unset.
        assert_eq!(parse(args("--tab")), Command::Run(LaunchOptions::default()));
    }

    #[test]
    fn export_icon_option() {
        assert_eq!(parse(args("--export-icon icon.png 256")), Command::ExportIcon { path: "icon.png".into(), size: 256 });
        assert_eq!(parse(args("--export-icon icon.png")), Command::ExportIcon { path: "icon.png".into(), size: 1024 });
        assert_eq!(parse(args("--export-icon icon.png 99999")), Command::ExportIcon { path: "icon.png".into(), size: 1024 });
        assert_eq!(parse(args("--export-icon")), Command::Help);
    }

    #[test]
    fn window_size_follows_the_launch() {
        let shot = |tab: Option<&str>, settings| LaunchOptions { screenshot: Some("x.png".into()), tab: tab.map(String::from), settings };
        assert_eq!(window_size(&LaunchOptions::default()), [400.0, 820.0]);
        assert_eq!(window_size(&shot(None, true)), [1000.0, 700.0]);
        assert_eq!(window_size(&shot(Some("shelffloat"), false)), [372.0, 330.0]);
        // Opening Settings for real keeps the panel size; Settings gets its own window.
        assert_eq!(window_size(&LaunchOptions { settings: true, ..Default::default() }), [400.0, 820.0]);
    }
}
