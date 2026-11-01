// The installer of the live session. Which program it is (Calamares today, SLI
// later), where to find it and how to launch it comes from installer.json, so
// replacing the installer means replacing that file and not this code. When
// the file is missing the defaults below are Calamares, which is what 2.x
// launched.
//
// The NumLock part is ported from 2.x's utils/numlockx_manager.py: it edits
// Calamares's own configuration, so it belongs to the installer too.

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
            name: "Calamares".to_string(),
            paths: vec![
                "/usr/bin/calamares".to_string(),
                "/usr/local/bin/calamares".to_string(),
                "/usr/bin/calamares-qt5".to_string(),
            ],
            env: BTreeMap::from([("QT_STYLE_OVERRIDE".to_string(), "fusion".to_string())]),
            needs_root: true,
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

const PACKAGES_CONF: &str = "/etc/calamares/modules/packages.conf";
const SHELLPROCESS_CONF: &str = "/etc/calamares/modules/shellprocess.conf";
const SYSTEM_SCRIPT: &str = "/usr/local/bin/disable-numlockx";
const SETTINGS_CONF: &str = "/etc/calamares/settings.conf";

/// Moves a file written by this user over a root-owned one, as 2.x did.
fn pkexec_move(temp: &Path, target: &str) -> Result<(), String> {
    let status = Command::new("pkexec")
        .arg("mv")
        .arg(temp)
        .arg(target)
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("pkexec mv {target} failed"))
    }
}

#[tauri::command]
pub fn get_numlock_enabled() -> bool {
    match fs::read_to_string("/etc/lightdm/lightdm.conf") {
        Ok(content) => {
            for line in content.lines() {
                if line.contains("greeter-setup-script=/usr/bin/numlockx on") {
                    return !line.trim_start().starts_with('#');
                }
            }
            false
        }
        // When the file cannot be read the default behavior is enabled.
        Err(_) => true,
    }
}

#[tauri::command(async)]
pub fn set_numlock_enabled(enabled: bool) -> Result<(), String> {
    if enabled {
        enable_numlock()
    } else {
        disable_numlock()
    }
}

fn disable_numlock() -> Result<(), String> {
    let packages = fs::read_to_string(PACKAGES_CONF).map_err(|e| e.to_string())?;
    let mut lines: Vec<String> = packages.lines().map(|l| l.to_string()).collect();
    if let Some(index) = lines.iter().position(|l| l.contains("'soplos-packager'")) {
        lines.insert(index + 1, "      - 'numlockx'".to_string());
    }
    let temp = std::env::temp_dir().join("soplos-live-packages.conf");
    fs::write(&temp, lines.join("\n") + "\n").map_err(|e| e.to_string())?;
    pkexec_move(&temp, PACKAGES_CONF)?;

    if Path::new(SHELLPROCESS_CONF).exists() {
        let shell = fs::read_to_string(SHELLPROCESS_CONF).map_err(|e| e.to_string())?;
        let mut lines: Vec<String> = shell.lines().map(|l| l.to_string()).collect();
        if let Some(index) = lines.iter().position(|l| l.contains("script:")) {
            lines.insert(index + 1, format!("   - \"{SYSTEM_SCRIPT}\""));
        }
        let temp = std::env::temp_dir().join("soplos-live-shellprocess.conf");
        fs::write(&temp, lines.join("\n") + "\n").map_err(|e| e.to_string())?;
        pkexec_move(&temp, SHELLPROCESS_CONF)?;
    }
    Ok(())
}

fn enable_numlock() -> Result<(), String> {
    let packages = fs::read_to_string(PACKAGES_CONF).map_err(|e| e.to_string())?;
    let kept: Vec<&str> = packages.lines().filter(|l| !l.contains("'numlockx'")).collect();
    let temp = std::env::temp_dir().join("soplos-live-packages.conf");
    fs::write(&temp, kept.join("\n") + "\n").map_err(|e| e.to_string())?;
    pkexec_move(&temp, PACKAGES_CONF)?;

    let module = "---\n\
# Shellprocess module for managing numlockx\n\
type: job\n\
name: shellprocess@manage_numlockx\n\
command: \"/etc/calamares/scripts/manage-numlockx.sh\"\n\
configurations:\n\
  - exec: /etc/calamares/scripts/manage-numlockx.sh enable\n\
    timeout: 10\n\
    chroot: true\n";
    let temp = std::env::temp_dir().join("soplos-live-shellprocess-numlockx.conf");
    fs::write(&temp, module).map_err(|e| e.to_string())?;
    pkexec_move(&temp, "/etc/calamares/modules/shellprocess_manage_numlockx.conf")?;

    // Make sure the module is part of the sequence.
    let settings = fs::read_to_string(SETTINGS_CONF).map_err(|e| e.to_string())?;
    let module_name = "shellprocess@manage_numlockx";
    if !settings.lines().any(|l| l.contains(module_name)) {
        let mut lines: Vec<String> = settings.lines().map(|l| l.to_string()).collect();
        if let Some(index) = lines.iter().position(|l| l.contains("- packages")) {
            lines.insert(index, format!("  - {module_name}"));
        }
        let temp = std::env::temp_dir().join("soplos-live-settings.conf");
        fs::write(&temp, lines.join("\n") + "\n").map_err(|e| e.to_string())?;
        pkexec_move(&temp, SETTINGS_CONF)?;
    }
    Ok(())
}
