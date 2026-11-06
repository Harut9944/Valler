//! Build script: on Windows, embeds the app icon and version details into
//! valler.exe so Explorer, the taskbar and Start show the Valler icon.
//! Other systems need nothing here.

fn main() {
    println!("cargo:rerun-if-changed=assets/valler.ico");
    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/valler.ico")
            .set("ProductName", "Valler")
            .set("FileDescription", "Valler — everyday desktop utilities")
            .set("LegalCopyright", "GPL-3.0-or-later");
        if let Err(e) = res.compile() {
            // A missing resource compiler shouldn't break the build; the window
            // still sets its icon at run time.
            println!("cargo:warning=Valler icon not embedded in the .exe: {e}");
        }
    }
}
