// Edition, desktop environment and display protocol detection, ported from
// 2.x's core/environment.py (theme detection dropped: this app tracks its own
// light/dark theme in the frontend, same as the rest of the 3.0 apps).

use serde::Serialize;
use std::fs;

#[derive(Serialize)]
pub struct EnvironmentInfo {
    #[serde(rename = "desktopEnvironment")]
    pub desktop_environment: String,
    #[serde(rename = "displayProtocol")]
    pub display_protocol: String,
    /// "tyron" (Xfce), "tyson" (Plasma), "boro" (GNOME) or "unknown".
    pub edition: String,
}

pub fn desktop_environment() -> String {
    let current_desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default().to_lowercase();
    if current_desktop.contains("gnome") {
        "gnome"
    } else if current_desktop.contains("kde") || current_desktop.contains("plasma") {
        "kde"
    } else if current_desktop.contains("xfce") {
        "xfce"
    } else if std::env::var("GNOME_DESKTOP_SESSION_ID").is_ok() {
        "gnome"
    } else if std::env::var("KDE_SESSION_VERSION").is_ok() {
        "kde"
    } else if std::env::var("XFCE_PANEL_MIGRATE_DEFAULT").is_ok() {
        "xfce"
    } else {
        "unknown"
    }
    .to_string()
}

pub fn display_protocol() -> String {
    let session_type = std::env::var("XDG_SESSION_TYPE").unwrap_or_default().to_lowercase();
    if session_type == "wayland" {
        "wayland"
    } else if session_type == "x11" || std::env::var("DISPLAY").is_ok() {
        "x11"
    } else {
        "unknown"
    }
    .to_string()
}

/// The edition file wins; without it the desktop decides, as in 2.x.
pub fn edition() -> String {
    if let Ok(content) = fs::read_to_string("/etc/soplos-edition") {
        let content = content.trim().to_lowercase();
        for name in ["tyron", "tyson", "boro"] {
            if content.contains(name) {
                return name.to_string();
            }
        }
        return "unknown".to_string();
    }
    match desktop_environment().as_str() {
        "xfce" => "tyron",
        "kde" => "tyson",
        "gnome" => "boro",
        _ => "unknown",
    }
    .to_string()
}

#[tauri::command]
pub fn get_environment_info() -> EnvironmentInfo {
    EnvironmentInfo {
        desktop_environment: desktop_environment(),
        display_protocol: display_protocol(),
        edition: edition(),
    }
}
