mod commands;
mod system_theme;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Must be the first plugin registered: it needs to intercept and
        // exit a second launch before anything else runs. Without it
        // nothing stops several windows from opening at once, each able to
        // mount and repair the same system.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            use tauri::Manager;
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            system_theme::get_system_theme,
            commands::environment::get_environment_info,
            commands::system::get_system_info,
            commands::system::check_internet,
            commands::system::get_autostart_enabled,
            commands::system::set_autostart_enabled,
            commands::installer::get_installer_info,
            commands::installer::launch_installer,
            commands::installer::get_install_checks,
            commands::installer::get_numlock_enabled,
            commands::installer::set_numlock_enabled,
            commands::tools::open_gparted,
            commands::tools::get_hardware_info,
            commands::tools::get_resolution_info,
            commands::tools::set_resolution,
            commands::language::get_language_state,
            commands::language::language_step,
            commands::rescue::rescue_scan,
            commands::rescue::rescue_mount,
            commands::rescue::rescue_unmount,
            commands::rescue::rescue_is_mounted,
            commands::rescue::rescue_step,
            commands::rescue::rescue_reboot,
            commands::rescue::rescue_open_terminal,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Soplos Welcome Live");
}
