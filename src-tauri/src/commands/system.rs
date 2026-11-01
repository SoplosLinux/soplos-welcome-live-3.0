// What the live session shows about the machine, the autostart switch of the
// welcome screen and the internet check of the status indicator. Ported from
// 2.x's main_window.py and utils/autostart.py.

use serde::Serialize;
use std::fs;
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

#[derive(Serialize)]
pub struct SystemInfo {
    pub cpu: String,
    pub memory: String,
    pub gpu: String,
    pub storage: String,
    pub uefi: bool,
}

#[tauri::command(async)]
pub fn get_system_info() -> SystemInfo {
    SystemInfo {
        cpu: detect_cpu(),
        memory: detect_memory(),
        gpu: detect_gpu(),
        storage: detect_storage(),
        uefi: is_uefi(),
    }
}

pub fn is_uefi() -> bool {
    fs::metadata("/sys/firmware/efi").is_ok()
}

fn detect_cpu() -> String {
    let content = fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    let name = content
        .lines()
        .find(|line| line.starts_with("model name"))
        .and_then(|line| line.split(':').nth(1))
        .map(|value| clean_cpu_name(value.trim()));
    name.unwrap_or_default()
}

/// Strips marketing noise from raw /proc/cpuinfo model names: integrated-GPU
/// suffixes ("w/ Radeon Graphics") duplicate the graphics card, and "(R)"/"(TM)"
/// / "CPU" only add clutter.
fn clean_cpu_name(raw: &str) -> String {
    let without_igpu = match raw.find(" w/ ") {
        Some(index) => &raw[..index],
        None => raw,
    };
    without_igpu
        .replace("(R)", "")
        .replace("(TM)", "")
        .replace(" CPU", "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn detect_memory() -> String {
    let content = fs::read_to_string("/proc/meminfo").unwrap_or_default();
    content
        .lines()
        .find(|line| line.starts_with("MemTotal"))
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|kb| kb.parse::<f64>().ok())
        .map(|kb| format!("{:.0} GB", (kb / (1024.0 * 1024.0)).round()))
        .unwrap_or_default()
}

fn detect_gpu() -> String {
    let Ok(output) = Command::new("lspci").output() else {
        return String::new();
    };
    let text = String::from_utf8_lossy(&output.stdout);
    let names: Vec<String> = text
        .lines()
        .filter(|line| line.contains("VGA compatible controller") || line.contains("3D controller"))
        .filter_map(|line| line.split(':').nth(2))
        .map(|value| value.trim().to_string())
        .collect();
    names.join(" · ")
}

fn detect_storage() -> String {
    let Ok(output) = Command::new("lsblk")
        .args(["-d", "-n", "-b", "-o", "NAME,SIZE,TYPE"])
        .output()
    else {
        return String::new();
    };
    let text = String::from_utf8_lossy(&output.stdout);
    let sizes: Vec<f64> = text
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 3 && parts[2] == "disk" && !parts[0].starts_with("zram") {
                parts[1].parse::<f64>().ok()
            } else {
                None
            }
        })
        .collect();
    if sizes.is_empty() {
        return String::new();
    }
    let total: f64 = sizes.iter().sum();
    format!("{:.0} GB", total / 1_000_000_000.0)
}

/// The same probe as the status indicator of 2.x: a TCP connection to 8.8.8.8:53
/// with a three-second timeout.
#[tauri::command(async)]
pub fn check_internet() -> bool {
    let address: SocketAddr = "8.8.8.8:53".parse().expect("static address");
    TcpStream::connect_timeout(&address, Duration::from_secs(3)).is_ok()
}

const AUTOSTART_DESKTOP_ENTRY: &str = "\
[Desktop Entry]
Type=Application
Version=1.0
Name=Soplos Welcome Live
Comment=Welcome to Soplos Linux Live Session
Exec=soplos-welcome-live
Icon=org.soplos.welcomelive
Terminal=false
Categories=System;Utility;
StartupNotify=true
X-GNOME-Autostart-enabled=true
X-KDE-autostart-after=panel
X-XFCE-Autostart-Override=true
";

fn autostart_file_path() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|_| "HOME is not set".to_string())?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("autostart")
        .join("org.soplos.welcomelive.desktop"))
}

#[tauri::command]
pub fn get_autostart_enabled() -> Result<bool, String> {
    let path = autostart_file_path()?;
    let Ok(content) = fs::read_to_string(&path) else {
        return Ok(false);
    };
    Ok(!content.contains("Hidden=true") && !content.contains("X-GNOME-Autostart-enabled=false"))
}

#[tauri::command]
pub fn set_autostart_enabled(enabled: bool) -> Result<(), String> {
    let path = autostart_file_path()?;
    if enabled {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(&path, AUTOSTART_DESKTOP_ENTRY).map_err(|e| e.to_string())?;
    } else if path.exists() {
        fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}
