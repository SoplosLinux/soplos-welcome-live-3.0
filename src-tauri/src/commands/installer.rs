// The installer of the live session. Which program it is (Soplos Linux
// Installer since 3.0; Calamares before), where to find it and how to launch
// it comes from installer.json, so replacing the installer means replacing
// that file and not this code. When the file is missing the defaults below
// are Soplos Linux Installer.
//
// The NumLock choice (Tyron) belongs to the installer too: it is left as a
// mark in the live session that the installer reads (see below).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use super::system::is_uefi;

const CONFIG_PATH: &str = "/usr/share/soplos-welcome-live/installer.json";

#[derive(Serialize, Deserialize, Clone)]
pub struct InstallerConfig {
    /// Shown in the interface ("Calamares", "SLI").
    pub name: String,
    /// The first of these that exists is the one launched.
    pub paths: Vec<String>,
    /// Environment variables for the installer process.
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    /// Launch through sudo (the live user has it without a password).
    #[serde(default = "default_true")]
    pub needs_root: bool,
}

fn default_true() -> bool {
    true
}

impl Default for InstallerConfig {
    fn default() -> Self {
        InstallerConfig {
            name: "Soplos Linux Installer".to_string(),
            paths: vec!["/usr/bin/soplos-linux-installer".to_string()],
            env: BTreeMap::new(),
            // It runs as the live user and raises only its own helper.
            needs_root: false,
        }
    }
}

fn load_config() -> InstallerConfig {
    fs::read_to_string(CONFIG_PATH)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

fn find_binary(config: &InstallerConfig) -> Option<String> {
    config.paths.iter().find(|path| Path::new(path.as_str()).exists()).cloned()
}

#[derive(Serialize)]
pub struct InstallerInfo {
    pub name: String,
    pub available: bool,
}

#[tauri::command]
pub fn get_installer_info() -> InstallerInfo {
    let config = load_config();
    let available = find_binary(&config).is_some();
    InstallerInfo { name: config.name, available }
}

#[tauri::command]
pub fn launch_installer() -> Result<(), String> {
    let config = load_config();
    let binary = find_binary(&config).ok_or_else(|| "installer-not-found".to_string())?;

    let mut command = if config.needs_root {
        let mut c = Command::new("sudo");
        c.arg("-n").arg("env");
        for (key, value) in &config.env {
            c.arg(format!("{key}={value}"));
        }
        c.arg(&binary);
        c
    } else {
        let mut c = Command::new(&binary);
        for (key, value) in &config.env {
            c.env(key, value);
        }
        c
    };
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[derive(Serialize)]
pub struct DiskInfo {
    pub name: String,
    pub size: String,
}

#[derive(Serialize)]
pub struct InstallChecks {
    pub uefi: bool,
    pub disks: Vec<DiskInfo>,
    /// None when the machine has no battery, Some(true) when it runs on it.
    #[serde(rename = "onBattery")]
    pub on_battery: Option<bool>,
}

#[tauri::command(async)]
pub fn get_install_checks() -> InstallChecks {
    InstallChecks { uefi: is_uefi(), disks: list_disks(), on_battery: battery_state() }
}

fn list_disks() -> Vec<DiskInfo> {
    let Ok(output) = Command::new("lsblk").args(["-d", "-n", "-p", "-o", "NAME,SIZE,TYPE"]).output() else {
        return Vec::new();
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 && parts[2] == "disk" && !parts[0].contains("/zram") {
                Some(DiskInfo { name: parts[0].to_string(), size: parts[1].to_string() })
            } else {
                None
            }
        })
        .collect()
}

fn battery_state() -> Option<bool> {
    let entries = fs::read_dir("/sys/class/power_supply").ok()?;
    let mut has_battery = false;
    let mut mains_online = false;
    for entry in entries.flatten() {
        let kind = fs::read_to_string(entry.path().join("type")).unwrap_or_default();
        match kind.trim() {
            "Battery" => has_battery = true,
            "Mains" => {
                let online = fs::read_to_string(entry.path().join("online")).unwrap_or_default();
                if online.trim() == "1" {
                    mains_online = true;
                }
            }
            _ => {}
        }
    }
    if has_battery {
        Some(!mains_online)
    } else {
        None
    }
}

// === NumLock on the installed system (Tyron only) ===
//
// 2.x and the first 3.0 versions edited the configuration of Calamares. Soplos
// Linux Installer reads a mark instead: when it is in the live session, the
// installer comments out `greeter-setup-script=/usr/bin/numlockx on` in the
// lightdm.conf of the installed system and purges numlockx, as the Tyron
// Calamares did (disable-numlockx and packages.conf). The live session runs
// in memory, so the mark goes with it.

const NUMLOCK_OFF_MARK: &str = "/etc/soplos-linux-installer/numlock-off";

#[tauri::command]
pub fn get_numlock_enabled() -> bool {
    !Path::new(NUMLOCK_OFF_MARK).exists()
}

#[tauri::command(async)]
pub fn set_numlock_enabled(enabled: bool) -> Result<(), String> {
    let run = |args: &[&str]| -> Result<(), String> {
        let status = Command::new("sudo").arg("-n").args(args).stdin(Stdio::null()).status().map_err(|e| e.to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("sudo {} failed", args.join(" ")))
        }
    };
    if enabled {
        run(&["rm", "-f", NUMLOCK_OFF_MARK])
    } else {
        let dir = Path::new(NUMLOCK_OFF_MARK).parent().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default();
        run(&["mkdir", "-p", &dir])?;
        run(&["touch", NUMLOCK_OFF_MARK])
    }
}
