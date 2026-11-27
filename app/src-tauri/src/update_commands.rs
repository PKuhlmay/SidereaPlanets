//! Support for the auto-updater (PLANETS-13). The plugin itself does the check, download and install; the one thing the window
//! needs to ask is whether this installation can replace itself.

/// Whether "Install and Restart" can work for this installation. On Linux only an AppImage replaces itself; a `.deb` is owned
/// by the package manager, so the window offers the download page instead. Windows (installer) and macOS (app bundle) always can.
#[tauri::command]
pub fn update_install_supported() -> bool {
    install_supported(
        cfg!(target_os = "linux"),
        std::env::var_os("APPIMAGE").is_some(),
    )
}

fn install_supported(linux: bool, appimage: bool) -> bool {
    !linux || appimage
}

#[cfg(test)]
mod tests {
    use super::install_supported;

    #[test]
    fn only_a_linux_install_outside_an_appimage_cannot_update_itself() {
        assert!(install_supported(false, false));
        assert!(install_supported(true, true));
        assert!(!install_supported(true, false));
    }
}
