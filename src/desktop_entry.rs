//! Linux taskbar and launcher icon. On Wayland (GNOME, KDE) the taskbar takes
//! a window's icon from the .desktop file matching its app id, not from the
//! window itself, so Valler installs a small launcher entry and icon for the
//! current user the first time it runs (or when the program moved).

#[cfg(target_os = "linux")]
pub fn ensure() {
    let Some(data) = dirs::data_dir() else { return };
    let Ok(exe) = std::env::current_exe() else { return };
    let apps = data.join("applications");
    let icons = data.join("icons/hicolor/256x256/apps");
    let desktop = apps.join("valler.desktop");
    let icon = icons.join("valler.png");
    let exec = exe.to_string_lossy().replace(' ', "\\s");
    let entry = format!(
        "[Desktop Entry]\nType=Application\nName=Valler\nComment=Everyday desktop utilities\nExec={exec}\nIcon=valler\nTerminal=false\nCategories=Utility;System;Monitor;\nStartupWMClass=valler\nKeywords=monitor;clipboard;keep awake;mixer;\n"
    );
    if std::fs::read_to_string(&desktop).ok().as_deref() != Some(entry.as_str()) {
        let _ = std::fs::create_dir_all(&apps);
        let _ = std::fs::write(&desktop, entry);
    }
    if !icon.exists() {
        let _ = std::fs::create_dir_all(&icons);
        let px = 256u32;
        let rgba = crate::ui::icon_rgba(px, None);
        let _ = image::save_buffer(&icon, &rgba, px, px, image::ExtendedColorType::Rgba8);
    }
}

#[cfg(not(target_os = "linux"))]
pub fn ensure() {}
