//! Where the person is in the app: the open Utilities tool and Settings page,
//! and the ids used by `--tab`.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sub {
    Clipboard,
    Snippets,
    Scratchpad,
    UrlClean,
    QuitApps,
    Ports,
    Cleaner,
    CommandBar,
    QuickToggles,
    CleaningMode,
    Timer,
    Packages,
    Uninstaller,
    AppUpdates,
    Media,
    Shelf,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SettingsPage {
    General,
    Features,
    Monitor,
    Clipboard,
    Snippets,
    UrlClean,
    KeepAwake,
    About,
}

pub fn sub_from_id(id: &str) -> Option<Sub> {
    Some(match id {
        "clipboard" => Sub::Clipboard,
        "snippets" => Sub::Snippets,
        "scratchpad" => Sub::Scratchpad,
        "cleanurl" => Sub::UrlClean,
        "quitapps" => Sub::QuitApps,
        "ports" => Sub::Ports,
        "cleaner" => Sub::Cleaner,
        "commandbar" => Sub::CommandBar,
        "toggles" => Sub::QuickToggles,
        "cleaning" => Sub::CleaningMode,
        "timer" => Sub::Timer,
        "packages" => Sub::Packages,
        "updates" => Sub::AppUpdates,
        "uninstaller" => Sub::Uninstaller,
        "media" => Sub::Media,
        "shelf" => Sub::Shelf,
        _ => return None,
    })
}

pub fn sub_title(s: Sub) -> &'static str {
    match s {
        Sub::CommandBar => "Command bar",
        Sub::QuickToggles => "Quick toggles",
        Sub::CleaningMode => "Cleaning mode",
        Sub::Timer => "Timer",
        Sub::Packages => "Package manager",
        Sub::Uninstaller => "Uninstaller",
        Sub::AppUpdates => "App updates",
        Sub::Media => "Media",
        Sub::Shelf => "Shelf",
        Sub::Clipboard => "Clipboard",
        Sub::Snippets => "Snippets",
        Sub::Scratchpad => "Scratchpad",
        Sub::UrlClean => "Clean URL",
        Sub::QuitApps => "Quit apps",
        Sub::Ports => "Port manager",
        Sub::Cleaner => "Cleaner",
    }
}
