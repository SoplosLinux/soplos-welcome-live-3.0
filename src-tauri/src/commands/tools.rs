// GParted, the hardware summary and the screen resolution of the live session.
// Ported from 2.x's main_window.py and utils/display_manager.py. The
// resolution works on X11 (xrandr) and on Plasma (kscreen-doctor); the GNOME
// route, which 2.x did through the Mutter D-Bus service, is not ported yet.

use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use super::environment::{desktop_environment, display_protocol};

#[tauri::command]
pub fn open_gparted() -> Result<(), String> {
    Command::new("pkexec")
        .arg("/usr/sbin/gparted")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[derive(Serialize)]
pub struct HardwareDisk {
    pub name: String,
    pub size: String,
    pub model: String,
}

#[derive(Serialize)]
pub struct HardwareInfo {
    pub cpu: String,
    pub memory: String,
    pub disks: Vec<HardwareDisk>,
}

#[tauri::command(async)]
pub fn get_hardware_info() -> HardwareInfo {
    let cpuinfo = fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    let cpu = cpuinfo
        .lines()
        .find(|line| line.starts_with("model name"))
        .and_then(|line| line.split_once(':'))
        .map(|(_, value)| value.trim().to_string())
        .unwrap_or_default();

    let meminfo = fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let memory = meminfo
        .lines()
        .find(|line| line.starts_with("MemTotal"))
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|kb| kb.parse::<f64>().ok())
        .map(|kb| format!("{:.1} GB", kb / (1024.0 * 1024.0)))
        .unwrap_or_default();

    let mut disks = Vec::new();
    if let Ok(output) = Command::new("lsblk").args(["-d", "-o", "NAME,SIZE,MODEL", "--noheadings"]).output() {
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let line = line.trim();
            if line.is_empty() || line.contains("loop") || line.starts_with("sr") || line.contains("rom") {
                continue;
            }
            let mut parts = line.splitn(3, char::is_whitespace);
            let name = parts.next().unwrap_or("").to_string();
            let size = parts.next().unwrap_or("").trim().to_string();
            let model = parts.next().unwrap_or("").trim().to_string();
            disks.push(HardwareDisk { name, size, model });
        }
    }
    HardwareInfo { cpu, memory, disks }
}

// === Screen resolution ===

#[derive(Serialize)]
pub struct ResolutionInfo {
    pub available: Vec<String>,
    pub current: Option<String>,
    /// False when this desktop has no supported route to change it.
    pub supported: bool,
}

fn area(resolution: &str) -> u64 {
    let mut parts = resolution.split('x');
    let w = parts.next().and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
    let h = parts.next().and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
    w * h
}

fn sorted_unique(mut list: Vec<String>) -> Vec<String> {
    list.sort();
    list.dedup();
    list.sort_by(|a, b| area(b).cmp(&area(a)));
    list
}

fn run_text(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        None
    }
}

/// "1920x1080" in a line of xrandr output, if any.
fn mode_in_line(line: &str) -> Option<String> {
    for token in line.split_whitespace() {
        let mut parts = token.split('x');
        if let (Some(w), Some(h), None) = (parts.next(), parts.next(), parts.next()) {
            if !w.is_empty() && !h.is_empty() && w.chars().all(|c| c.is_ascii_digit()) && h.chars().all(|c| c.is_ascii_digit()) {
                return Some(token.to_string());
            }
        }
    }
    None
}

fn x11_info() -> ResolutionInfo {
    let text = run_text("xrandr", &["-q"]).unwrap_or_default();
    let mut available = Vec::new();
    let mut current = None;
    for line in text.lines() {
        if !line.starts_with(char::is_whitespace) {
            continue;
        }
        if let Some(mode) = mode_in_line(line) {
            if line.contains('*') && current.is_none() {
                current = Some(mode.clone());
            }
            available.push(mode);
        }
    }
    ResolutionInfo { available: sorted_unique(available), current, supported: true }
}

fn x11_connected_output() -> Option<String> {
    let text = run_text("xrandr", &["-q"])?;
    text.lines()
        .find(|line| line.contains(" connected"))
        .and_then(|line| line.split_whitespace().next())
        .map(|name| name.to_string())
}

fn x11_set(resolution: &str) -> Result<(), String> {
    let output = x11_connected_output().ok_or_else(|| "no connected output".to_string())?;
    let status = Command::new("xrandr")
        .args(["--output", &output, "--mode", resolution])
        .status()
        .map_err(|e| e.to_string())?;
    if !status.success() {
        return Err("xrandr failed".to_string());
    }
    persist_x11(&output, resolution);
    Ok(())
}

/// Keeps the resolution through the session restart the language change does.
fn persist_x11(output: &str, mode: &str) {
    let Ok(home) = std::env::var("HOME") else { return };
    let dir = PathBuf::from(home).join(".config").join("autostart");
    if fs::create_dir_all(&dir).is_err() {
        return;
    }
    let exec = if desktop_environment() == "xfce" {
        // Xfce needs xfdesktop restarted for the wallpaper to follow the change.
        format!("sh -c 'xrandr --output {output} --mode {mode}; sleep 2; xfdesktop --quit; sleep 1; xfdesktop &'")
    } else {
        format!("xrandr --output {output} --mode {mode}")
    };
    let content = format!(
        "[Desktop Entry]\nType=Application\nName=Restore Resolution\nExec={exec}\nHidden=false\nNoDisplay=false\nX-GNOME-Autostart-enabled=true\n"
    );
    let _ = fs::write(dir.join("soplos-resolution.desktop"), content);
}

fn kde_json() -> Option<Value> {
    let text = run_text("kscreen-doctor", &["-j"])?;
    serde_json::from_str(&text).ok()
}

fn size_of(mode: &Value) -> Option<(u64, u64)> {
    let size = mode.get("size")?;
    Some((size.get("width")?.as_u64()?, size.get("height")?.as_u64()?))
}

fn kde_info() -> ResolutionInfo {
    let Some(data) = kde_json() else {
        return ResolutionInfo { available: Vec::new(), current: None, supported: false };
    };
    let mut available = Vec::new();
    let mut current = None;
    for out in data.get("outputs").and_then(|v| v.as_array()).into_iter().flatten() {
        if !out.get("connected").and_then(|v| v.as_bool()).unwrap_or(false) {
            continue;
        }
        let current_id = out.get("currentModeId").map(|v| v.to_string().trim_matches('"').to_string());
        for mode in out.get("modes").and_then(|v| v.as_array()).into_iter().flatten() {
            if let Some((w, h)) = size_of(mode) {
                let label = format!("{w}x{h}");
                let id = mode.get("id").map(|v| v.to_string().trim_matches('"').to_string());
                if current.is_none() && current_id.is_some() && id == current_id {
                    current = Some(label.clone());
                }
                available.push(label);
            }
        }
        if current.is_none() {
            // No explicit current mode: the preferred one is the best guess.
            for mode in out.get("modes").and_then(|v| v.as_array()).into_iter().flatten() {
                if mode.get("preferred").and_then(|v| v.as_bool()).unwrap_or(false) {
                    if let Some((w, h)) = size_of(mode) {
                        current = Some(format!("{w}x{h}"));
                        break;
                    }
                }
            }
        }
    }
    ResolutionInfo { available: sorted_unique(available), current, supported: true }
}

fn kde_set(resolution: &str) -> Result<(), String> {
    let data = kde_json().ok_or_else(|| "kscreen-doctor is not available".to_string())?;
    let mut parts = resolution.split('x');
    let w: u64 = parts.next().and_then(|v| v.parse().ok()).ok_or("invalid resolution")?;
    let h: u64 = parts.next().and_then(|v| v.parse().ok()).ok_or("invalid resolution")?;

    let mut args: Vec<String> = Vec::new();
    for out in data.get("outputs").and_then(|v| v.as_array()).into_iter().flatten() {
        if !out.get("connected").and_then(|v| v.as_bool()).unwrap_or(false) {
            continue;
        }
        let Some(out_id) = out.get("id").map(|v| v.to_string().trim_matches('"').to_string()) else { continue };
        for mode in out.get("modes").and_then(|v| v.as_array()).into_iter().flatten() {
            if size_of(mode) == Some((w, h)) {
                if let Some(mode_id) = mode.get("id").map(|v| v.to_string().trim_matches('"').to_string()) {
                    args.push(format!("output.{out_id}.mode.{mode_id}"));
                    break;
                }
            }
        }
    }
    if args.is_empty() {
        return Err("resolution not available".to_string());
    }
    let status = Command::new("kscreen-doctor").args(&args).status().map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err("kscreen-doctor failed".to_string())
    }
}

#[tauri::command(async)]
pub fn get_resolution_info() -> ResolutionInfo {
    if display_protocol() == "x11" {
        return x11_info();
    }
    if desktop_environment() == "kde" {
        return kde_info();
    }
    ResolutionInfo { available: Vec::new(), current: None, supported: false }
}

#[tauri::command(async)]
pub fn set_resolution(resolution: String) -> Result<(), String> {
    // Only "<width>x<height>" reaches the commands below.
    let valid = resolution.split('x').count() == 2
        && resolution.split('x').all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()));
    if !valid {
        return Err("invalid resolution".to_string());
    }
    if display_protocol() == "x11" {
        return x11_set(&resolution);
    }
    if desktop_environment() == "kde" {
        return kde_set(&resolution);
    }
    Err("unsupported".to_string())
}
