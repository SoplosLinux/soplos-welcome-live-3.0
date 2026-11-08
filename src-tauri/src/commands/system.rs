// What the live session shows about the machine, the autostart switch of the
// welcome screen and the internet check of the status indicator. Ported from
// 2.x's main_window.py and utils/autostart.py.

use serde::Serialize;
use std::fs;
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

/// What the installer needs before it starts. It downloads packages from the
/// servers of Soplos, so without a connection to them, or with a clock that
/// does not match theirs, it stops with error 100.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Readiness {
    pub internet: bool,
    /// Seconds the clock of this computer is ahead of the servers (negative:
    /// behind). None when the servers did not say what time it is.
    pub clock_offset: Option<i64>,
}

/// A server of the repositories of Soplos: any answer of it proves that the
/// name resolves, that the connection works and that HTTPS is not blocked.
const PROBE_URL: &str = "https://raw.githubusercontent.com/SoplosLinux/soplos-kernels/main/";

/// The headers of the answer of the probe. None when there was no answer.
fn probe_headers() -> Option<String> {
    let output = Command::new("curl")
        .args(["-sI", "-m", "6", "--connect-timeout", "5", PROBE_URL])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    if output.status.success() && text.starts_with("HTTP/") {
        Some(text)
    } else {
        None
    }
}

/// Days from 1970-01-01 of a date of the civil calendar.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// The `Date:` header of an HTTP answer ("Sun, 08 Nov 2026 10:00:00 GMT") as
/// seconds since the epoch.
fn server_epoch(headers: &str) -> Option<i64> {
    let line = headers.lines().find(|l| l.to_ascii_lowercase().starts_with("date:"))?;
    let value = line.split_once(':')?.1.trim();
    let parts: Vec<&str> = value.split_whitespace().collect();
    // [weekday,] day month year time GMT
    let day: i64 = parts.get(1)?.parse().ok()?;
    let month_name = *parts.get(2)?;
    let month = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]
        .iter()
        .position(|m| m.eq_ignore_ascii_case(month_name))? as i64
        + 1;
    let year: i64 = parts.get(3)?.parse().ok()?;
    let mut time = parts.get(4)?.split(':');
    let hour: i64 = time.next()?.parse().ok()?;
    let minute: i64 = time.next()?.parse().ok()?;
    let second: i64 = time.next()?.parse().ok()?;
    Some(days_from_civil(year, month, day) * 86400 + hour * 3600 + minute * 60 + second)
}

fn now_epoch() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

#[tauri::command(async)]
pub fn check_readiness() -> Readiness {
    match probe_headers() {
        Some(headers) => Readiness { internet: true, clock_offset: server_epoch(&headers).map(|server| now_epoch() - server) },
        // Without curl or without an answer: the plain connection test of the
        // status indicator decides, and the clock is not known.
        None => Readiness { internet: check_internet(), clock_offset: None },
    }
}

/// Sets the clock from the time the servers report, writes it to the hardware
/// clock and turns on the network time. One sudo call, the script on its stdin.
#[tauri::command(async)]
pub fn fix_clock() -> Result<(), String> {
    let epoch = probe_headers().as_deref().and_then(server_epoch).ok_or_else(|| "the servers did not say what time it is".to_string())?;
    let script = format!(
        "#!/bin/bash\ndate -u -s @{epoch} >/dev/null || exit 1\nhwclock --systohc 2>/dev/null || true\ntimedatectl set-ntp true 2>/dev/null || true\n"
    );
    let mut child = Command::new("sudo")
        .args(["-n", "/bin/bash", "-s"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    child.stdin.take().ok_or_else(|| "no stdin".to_string())?.write_all(script.as_bytes()).map_err(|e| e.to_string())?;
    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_date_of_a_server() {
        let headers = "HTTP/2 200\r\ncontent-type: text/plain\r\nDate: Sun, 08 Nov 2026 10:00:00 GMT\r\n\r\n";
        assert_eq!(server_epoch(headers), Some(1_794_132_000));
        assert_eq!(server_epoch("HTTP/2 200\r\n\r\n"), None);
        // 1970-01-01 and a leap day
        assert_eq!(server_epoch("date: Thu, 01 Jan 1970 00:00:00 GMT"), Some(0));
        assert_eq!(server_epoch("date: Thu, 29 Feb 2024 12:00:00 GMT"), Some(1_709_208_000));
    }
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
