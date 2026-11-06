//! Launch at login: a login item on macOS, a Run registry entry on Windows
//! and an XDG autostart entry on Linux (works on GNOME and KDE).

fn handle() -> Result<auto_launch::AutoLaunch, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    auto_launch::AutoLaunchBuilder::new()
        .set_app_name("Valler")
        .set_app_path(&exe.to_string_lossy())
        .build()
        .map_err(|e| e.to_string())
}

pub fn is_enabled() -> bool {
    handle().and_then(|h| h.is_enabled().map_err(|e| e.to_string())).unwrap_or(false)
}

pub fn set(on: bool) -> Result<(), String> {
    let h = handle()?;
    if on { h.enable() } else { h.disable() }.map_err(|e| e.to_string())
}
