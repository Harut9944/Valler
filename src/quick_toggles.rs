//! Quick toggles: one-click system actions — dark mode, lock screen, display
//! off, screen saver and empty the trash — using each system's own tools.

use std::process::Command;
use std::sync::{Arc, Mutex};

pub type Outcome = Arc<Mutex<Option<String>>>;

fn run(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd).args(args).status().map(|s| s.success()).unwrap_or(false)
}

fn output(cmd: &str, args: &[&str]) -> Option<String> {
    let o = Command::new(cmd).args(args).output().ok()?;
    o.status.success().then(|| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

#[cfg(target_os = "linux")]
fn is_kde() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default().to_uppercase().contains("KDE")
}

/// Whether the desktop currently uses a dark appearance (None if unknown).
pub fn dark_mode() -> Option<bool> {
    #[cfg(target_os = "linux")]
    {
        if is_kde() {
            let s = output("kreadconfig6", &["--group", "General", "--key", "ColorScheme"])
                .or_else(|| output("kreadconfig5", &["--group", "General", "--key", "ColorScheme"]))?;
            return Some(s.to_lowercase().contains("dark"));
        }
        let s = output("gsettings", &["get", "org.gnome.desktop.interface", "color-scheme"])?;
        Some(s.contains("dark"))
    }
    #[cfg(target_os = "windows")]
    {
        let s = output("reg", &["query", r"HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize", "/v", "AppsUseLightTheme"])?;
        Some(s.contains("0x0"))
    }
    #[cfg(target_os = "macos")]
    {
        Some(output("defaults", &["read", "-g", "AppleInterfaceStyle"]).is_some_and(|s| s.contains("Dark")))
    }
}

pub fn set_dark_mode(dark: bool) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        let ok = if is_kde() {
            run("plasma-apply-colorscheme", &[if dark { "BreezeDark" } else { "BreezeLight" }])
        } else {
            let a = run("gsettings", &["set", "org.gnome.desktop.interface", "color-scheme", if dark { "prefer-dark" } else { "default" }]);
            // Older GTK apps follow the theme name rather than the color scheme.
            let _ = run("gsettings", &["set", "org.gnome.desktop.interface", "gtk-theme", if dark { "Adwaita-dark" } else { "Adwaita" }]);
            a
        };
        if ok { Ok(()) } else { Err("This desktop did not accept the change".into()) }
    }
    #[cfg(target_os = "windows")]
    {
        let v = if dark { "0" } else { "1" };
        let key = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
        let a = run("reg", &["add", key, "/v", "AppsUseLightTheme", "/t", "REG_DWORD", "/d", v, "/f"]);
        let b = run("reg", &["add", key, "/v", "SystemUsesLightTheme", "/t", "REG_DWORD", "/d", v, "/f"]);
        if a && b { Ok(()) } else { Err("Windows did not accept the change".into()) }
    }
    #[cfg(target_os = "macos")]
    {
        let script = format!("tell application \"System Events\" to tell appearance preferences to set dark mode to {dark}");
        if run("osascript", &["-e", &script]) { Ok(()) } else { Err("macOS did not allow the change (check Automation permission)".into()) }
    }
}

pub fn lock_screen() -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        let ok = run("loginctl", &["lock-session"])
            || run("xdg-screensaver", &["lock"])
            || run("dbus-send", &["--session", "--dest=org.freedesktop.ScreenSaver", "--type=method_call", "/org/freedesktop/ScreenSaver", "org.freedesktop.ScreenSaver.Lock"]);
        if ok { Ok(()) } else { Err("No screen locker answered".into()) }
    }
    #[cfg(target_os = "windows")]
    {
        if run("rundll32.exe", &["user32.dll,LockWorkStation"]) { Ok(()) } else { Err("Windows refused to lock".into()) }
    }
    #[cfg(target_os = "macos")]
    {
        // Sleeping the display locks when "require password" is on.
        if run("pmset", &["displaysleepnow"]) { Ok(()) } else { Err("macOS refused".into()) }
    }
}

pub fn display_off() -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        let wayland = std::env::var("WAYLAND_DISPLAY").is_ok();
        let ok = if is_kde() {
            run("kscreen-doctor", &["--dpms", "off"])
        } else if !wayland {
            run("xset", &["dpms", "force", "off"])
        } else {
            run("busctl", &["--user", "call", "org.gnome.Shell", "/org/gnome/ScreenSaver", "org.gnome.ScreenSaver", "SetActive", "b", "true"])
        };
        if ok { Ok(()) } else { Err("This desktop has no command to turn the display off".into()) }
    }
    #[cfg(target_os = "windows")]
    {
        let ps = "(Add-Type '[DllImport(\"user32.dll\")]public static extern int SendMessage(int h,int m,int w,int l);' -Name W -PassThru)::SendMessage(-1,0x0112,0xF170,2)";
        if run("powershell", &["-NoProfile", "-WindowStyle", "Hidden", "-Command", ps]) { Ok(()) } else { Err("Windows refused".into()) }
    }
    #[cfg(target_os = "macos")]
    {
        if run("pmset", &["displaysleepnow"]) { Ok(()) } else { Err("macOS refused".into()) }
    }
}

pub fn screen_saver() -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        let ok = run("xdg-screensaver", &["activate"])
            || run("dbus-send", &["--session", "--dest=org.freedesktop.ScreenSaver", "--type=method_call", "/org/freedesktop/ScreenSaver", "org.freedesktop.ScreenSaver.SetActive", "boolean:true"]);
        if ok { Ok(()) } else { Err("No screen saver answered".into()) }
    }
    #[cfg(target_os = "windows")]
    {
        let ps = "(Add-Type '[DllImport(\"user32.dll\")]public static extern int SendMessage(int h,int m,int w,int l);' -Name S -PassThru)::SendMessage(-1,0x0112,0xF140,0)";
        if run("powershell", &["-NoProfile", "-WindowStyle", "Hidden", "-Command", ps]) { Ok(()) } else { Err("Windows refused".into()) }
    }
    #[cfg(target_os = "macos")]
    {
        if run("open", &["-a", "ScreenSaverEngine"]) { Ok(()) } else { Err("macOS refused".into()) }
    }
}

/// Number of items in the trash, when the system lets us count them.
pub fn trash_count() -> Option<usize> {
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    {
        trash::os_limited::list().ok().map(|l| l.len())
    }
    #[cfg(target_os = "macos")]
    {
        None
    }
}

pub fn empty_trash() -> Result<usize, String> {
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    {
        let items = trash::os_limited::list().map_err(|e| e.to_string())?;
        let n = items.len();
        trash::os_limited::purge_all(items).map_err(|e| e.to_string())?;
        Ok(n)
    }
    #[cfg(target_os = "macos")]
    {
        if run("osascript", &["-e", "tell application \"Finder\" to empty trash"]) { Ok(0) } else { Err("Finder refused (check Automation permission)".into()) }
    }
}

/// Runs an action on a background thread and leaves a message for the UI.
pub fn spawn(out: Outcome, ctx: egui::Context, f: impl FnOnce() -> String + Send + 'static) {
    std::thread::spawn(move || {
        let msg = f();
        if let Ok(mut o) = out.lock() {
            *o = Some(msg);
        }
        ctx.request_repaint();
    });
}

#[allow(dead_code)]
fn _unused() {
    let _ = output;
}
