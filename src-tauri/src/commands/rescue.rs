// System rescue: find the Linux systems installed on the machine, mount one in
// /mnt/chroot and repair it from there. Ported from 2.x's
// core/chroot_operations.py. What changes is how the system is chosen: 2.x
// asked the user for the root, boot, EFI and home partitions; here every
// partition is looked at and the mount points are read from the installed
// system's own /etc/fstab, so the user only picks a system (or is not asked
// at all when there is one).
//
// Everything privileged goes through `sudo -n`, as in 2.x: the live user has
// it without a password.

use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use super::environment::desktop_environment;

const MOUNT_POINT: &str = "/mnt/chroot";

/// Filesystems a Linux root can be on.
const ROOT_FILESYSTEMS: [&str; 7] = ["ext4", "ext3", "ext2", "xfs", "btrfs", "f2fs", "jfs"];

#[derive(Serialize, Clone)]
pub struct FoundSystem {
    pub device: String,
    pub name: String,
    pub size: String,
    pub fstype: String,
    /// The btrfs subvolume that holds the root, when there is one.
    pub subvol: Option<String>,
    /// The disk it lives on, e.g. "Samsung SSD 980 · 500G".
    pub disk: String,
    pub users: Vec<String>,
}

/// Runs a command, returning its stdout, or its stderr as the error.
fn run(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program).args(args).stdin(Stdio::null()).output().map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let out = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Err(if err.is_empty() { out } else { err })
    }
}

fn sudo(args: &[&str]) -> Result<String, String> {
    let mut full = vec!["-n"];
    full.extend_from_slice(args);
    run("sudo", &full)
}

fn is_mountpoint(path: &str) -> bool {
    Command::new("mountpoint")
        .args(["-q", path])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn string_of(value: &Value, key: &str) -> String {
    value.get(key).and_then(|v| v.as_str()).unwrap_or("").to_string()
}

// === Finding the installed systems ===

#[tauri::command(async)]
pub fn rescue_scan() -> Result<Vec<FoundSystem>, String> {
    let json = run("lsblk", &["-J", "-p", "-o", "NAME,SIZE,FSTYPE,MOUNTPOINT,LABEL,TYPE,MODEL"])?;
    let data: Value = serde_json::from_str(&json).map_err(|e| e.to_string())?;

    let mut found = Vec::new();
    for disk in data.get("blockdevices").and_then(|v| v.as_array()).into_iter().flatten() {
        if string_of(disk, "type") != "disk" || string_of(disk, "name").contains("/zram") {
            continue;
        }
        let model = string_of(disk, "model");
        let disk_label = if model.trim().is_empty() {
            string_of(disk, "size")
        } else {
            format!("{} · {}", model.trim(), string_of(disk, "size"))
        };
        for part in disk.get("children").and_then(|v| v.as_array()).into_iter().flatten() {
            let fstype = string_of(part, "fstype").to_lowercase();
            if !ROOT_FILESYSTEMS.contains(&fstype.as_str()) {
                continue;
            }
            // The live system's own medium is not an installed system.
            if string_of(part, "mountpoint") == "/" {
                continue;
            }
            let device = string_of(part, "name");
            if let Some(mut system) = probe_partition(&device, &fstype) {
                system.size = string_of(part, "size");
                system.disk = disk_label.clone();
                if system.name.is_empty() {
                    system.name = string_of(part, "label");
                }
                found.push(system);
            }
        }
    }
    Ok(found)
}

/// Mounts a partition read-only for a moment and reads what system it holds.
fn probe_partition(device: &str, fstype: &str) -> Option<FoundSystem> {
    let mut subvols: Vec<Option<&str>> = vec![None];
    if fstype == "btrfs" {
        subvols.push(Some("@"));
    }
    for subvol in subvols {
        let temp = format!("/tmp/soplos-live-probe-{}", std::process::id());
        if sudo(&["mkdir", "-p", &temp]).is_err() {
            return None;
        }
        let mounted = match subvol {
            Some(name) => sudo(&["mount", "-t", "btrfs", "-o", &format!("ro,subvol={name}"), device, &temp]),
            None => sudo(&["mount", "-o", "ro", device, &temp]),
        };
        let result = if mounted.is_ok() { read_system(Path::new(&temp)) } else { None };
        if mounted.is_ok() {
            let _ = sudo(&["umount", &temp]);
        }
        let _ = sudo(&["rmdir", &temp]);
        if let Some((name, users)) = result {
            return Some(FoundSystem {
                device: device.to_string(),
                name,
                size: String::new(),
                fstype: fstype.to_string(),
                subvol: subvol.map(|s| s.to_string()),
                disk: String::new(),
                users,
            });
        }
    }
    None
}

/// The name and the human users of a mounted root, or None when it is not one.
fn read_system(root: &Path) -> Option<(String, Vec<String>)> {
    let passwd = fs::read_to_string(root.join("etc/passwd")).ok()?;
    if !root.join("etc/fstab").exists() {
        return None;
    }
    let os_release = fs::read_to_string(root.join("etc/os-release")).unwrap_or_default();
    let name = os_release
        .lines()
        .find_map(|line| line.strip_prefix("PRETTY_NAME="))
        .map(|value| value.trim_matches('"').to_string())
        .unwrap_or_else(|| "Linux".to_string());

    let mut users = Vec::new();
    for line in passwd.lines() {
        let fields: Vec<&str> = line.split(':').collect();
        if fields.len() < 7 {
            continue;
        }
        let uid: u32 = fields[2].parse().unwrap_or(0);
        let shell = fields[6];
        if (1000..60000).contains(&uid) && !shell.ends_with("nologin") && !shell.ends_with("false") {
            users.push(fields[0].to_string());
        }
    }
    Some((name, users))
}

// === Mounting ===

fn unmount_everything() {
    for sub in ["dev/pts", "dev", "proc", "sys", "boot/efi", "efi", "boot", "home"] {
        let path = format!("{MOUNT_POINT}/{sub}");
        if is_mountpoint(&path) {
            let _ = sudo(&["umount", "-l", &path]);
        }
    }
    if is_mountpoint(MOUNT_POINT) {
        let _ = sudo(&["umount", "-l", MOUNT_POINT]);
    }
}

#[tauri::command(async)]
pub fn rescue_unmount() {
    unmount_everything();
}

#[tauri::command]
pub fn rescue_is_mounted() -> bool {
    is_mountpoint(MOUNT_POINT)
}

/// The device a /etc/fstab spec ("UUID=...", "LABEL=...", "/dev/...") names.
fn resolve_spec(spec: &str) -> Option<String> {
    if spec.starts_with('/') {
        return Some(spec.to_string());
    }
    if spec.contains('=') {
        return run("findfs", &[spec]).ok().map(|s| s.trim().to_string());
    }
    None
}

#[tauri::command(async)]
pub fn rescue_mount(device: String, subvol: Option<String>) -> Result<String, String> {
    let mut log = String::new();
    unmount_everything();
    sudo(&["mkdir", "-p", MOUNT_POINT])?;

    let root_result = match &subvol {
        Some(name) => sudo(&["mount", "-t", "btrfs", "-o", &format!("subvol={name}"), &device, MOUNT_POINT]),
        None => sudo(&["mount", &device, MOUNT_POINT]),
    };
    if let Err(e) = root_result {
        unmount_everything();
        return Err(format!("{device}: {e}"));
    }
    log.push_str(&format!("mounted {device} on {MOUNT_POINT}\n"));

    // The other partitions come from the installed system's own fstab.
    let fstab = fs::read_to_string(format!("{MOUNT_POINT}/etc/fstab")).unwrap_or_default();
    for wanted in ["/boot", "/boot/efi", "/efi", "/home"] {
        for line in fstab.lines().map(str::trim).filter(|l| !l.starts_with('#') && !l.is_empty()) {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() < 4 || fields[1] != wanted {
                continue;
            }
            let Some(source) = resolve_spec(fields[0]) else { continue };
            let target = format!("{MOUNT_POINT}{wanted}");
            let _ = sudo(&["mkdir", "-p", &target]);
            // Only the subvolume option matters for a btrfs mount here.
            let subvol_option = fields[3].split(',').find(|o| o.starts_with("subvol=")).map(|o| o.to_string());
            let result = match (fields[2], subvol_option) {
                ("btrfs", Some(option)) => sudo(&["mount", "-t", "btrfs", "-o", &option, &source, &target]),
                _ => sudo(&["mount", &source, &target]),
            };
            match result {
                Ok(_) => log.push_str(&format!("mounted {source} on {wanted}\n")),
                Err(e) => log.push_str(&format!("could not mount {wanted}: {e}\n")),
            }
            break;
        }
    }

    for (source, target) in [("/dev", "dev"), ("/proc", "proc"), ("/sys", "sys"), ("/dev/pts", "dev/pts")] {
        let path = format!("{MOUNT_POINT}/{target}");
        let _ = sudo(&["mkdir", "-p", &path]);
        if let Err(e) = sudo(&["mount", "--bind", source, &path]) {
            log.push_str(&format!("could not bind {source}: {e}\n"));
        }
    }

    if Path::new(&format!("{MOUNT_POINT}/etc")).exists() {
        let _ = sudo(&["cp", "/etc/resolv.conf", &format!("{MOUNT_POINT}/etc/resolv.conf")]);
    }

    if !is_valid_system() {
        unmount_everything();
        return Err("invalid-system".to_string());
    }
    Ok(log)
}

/// At least three essential files and a shell, as 2.x checked.
fn is_valid_system() -> bool {
    let essential = ["bin", "usr/bin", "lib", "usr/lib", "etc/fstab", "etc/passwd", "etc/shadow"];
    let found = essential.iter().filter(|p| Path::new(&format!("{MOUNT_POINT}/{p}")).exists()).count();
    let shell = ["bin/bash", "bin/sh", "usr/bin/bash", "usr/bin/sh"]
        .iter()
        .any(|p| Path::new(&format!("{MOUNT_POINT}/{p}")).exists());
    found >= 3 && shell
}

// === The repairs ===

/// /dev/sda4 -> /dev/sda, /dev/nvme0n1p2 -> /dev/nvme0n1
fn parent_disk(partition: &str) -> String {
    for prefix in ["/dev/nvme", "/dev/mmcblk"] {
        if let Some(rest) = partition.strip_prefix(prefix) {
            if let Some(index) = rest.rfind('p') {
                return format!("{prefix}{}", &rest[..index]);
            }
        }
    }
    let trimmed = partition.trim_end_matches(|c: char| c.is_ascii_digit());
    if trimmed.len() < "/dev/".len() {
        partition.to_string()
    } else {
        trimmed.to_string()
    }
}

fn in_chroot(args: &[&str]) -> Result<String, String> {
    let mut full = vec!["chroot", MOUNT_POINT];
    full.extend_from_slice(args);
    sudo(&full)
}

fn valid_username(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() || c == '_' => {}
        _ => return false,
    }
    name.len() <= 32 && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

/// One repair step. The frontend calls them in order so its progress shows
/// real work: "grub_install", "update_grub", "initramfs" or "password".
#[tauri::command(async)]
pub fn rescue_step(step: String, device: Option<String>, user: Option<String>, password: Option<String>) -> Result<String, String> {
    if !is_mountpoint(MOUNT_POINT) {
        return Err("not-mounted".to_string());
    }
    match step.as_str() {
        "grub_install" => {
            let efi = is_mountpoint(&format!("{MOUNT_POINT}/boot/efi")) || is_mountpoint(&format!("{MOUNT_POINT}/efi"));
            if efi {
                let efi_dir = if is_mountpoint(&format!("{MOUNT_POINT}/boot/efi")) { "/boot/efi" } else { "/efi" };
                in_chroot(&["grub-install", "--target=x86_64-efi", &format!("--efi-directory={efi_dir}"), "--recheck"])
            } else {
                let device = device.ok_or_else(|| "no device".to_string())?;
                in_chroot(&["grub-install", &parent_disk(&device)])
            }
        }
        "update_grub" => in_chroot(&["update-grub"]),
        "initramfs" => in_chroot(&["dracut", "--regenerate-all", "--force"]),
        "password" => {
            let user = user.ok_or_else(|| "no user".to_string())?;
            let password = password.ok_or_else(|| "no password".to_string())?;
            if !valid_username(&user) {
                return Err("invalid-user".to_string());
            }
            if password.is_empty() || password.contains('\n') || password.contains('\r') {
                return Err("invalid-password".to_string());
            }
            let mut child = Command::new("sudo")
                .args(["-n", "chroot", MOUNT_POINT, "chpasswd"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| e.to_string())?;
            child
                .stdin
                .take()
                .ok_or_else(|| "no stdin".to_string())?
                .write_all(format!("{user}:{password}\n").as_bytes())
                .map_err(|e| e.to_string())?;
            let output = child.wait_with_output().map_err(|e| e.to_string())?;
            if output.status.success() {
                Ok(String::new())
            } else {
                Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
            }
        }
        other => Err(format!("unknown step: {other}")),
    }
}

#[tauri::command]
pub fn rescue_reboot() -> Result<(), String> {
    Command::new("sudo")
        .args(["-n", "systemctl", "reboot"])
        .stdin(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

// === The terminal of the advanced route ===

fn shell_quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', "'\\''"))
}

#[tauri::command]
pub fn rescue_open_terminal(title: String, exit_message: String, cleanup_message: String, done_message: String) -> Result<(), String> {
    if !is_mountpoint(MOUNT_POINT) {
        return Err("not-mounted".to_string());
    }
    let mp = MOUNT_POINT;
    let separator = "=".repeat(34);
    let script = format!(
        r#"#!/bin/bash
echo "{separator}"
printf '%s\n' {title}
echo "{mp}"
printf '%s\n' {exit_message}
echo "{separator}"

if [ -x "{mp}/bin/bash" ]; then
    SHELL_CMD="/bin/bash"
elif [ -x "{mp}/usr/bin/bash" ]; then
    SHELL_CMD="/usr/bin/bash"
else
    SHELL_CMD="/bin/sh"
fi

sudo -n chroot {mp} $SHELL_CMD

printf '%s\n' {cleanup_message}
for d in dev/pts dev proc sys boot/efi efi boot home; do
    sudo -n umount -l {mp}/$d 2>/dev/null || true
done
sudo -n umount -l {mp} 2>/dev/null || true
printf '%s\n' {done_message}
"#,
        title = shell_quote(&title),
        exit_message = shell_quote(&exit_message),
        cleanup_message = shell_quote(&cleanup_message),
        done_message = shell_quote(&done_message),
    );

    let script_path: PathBuf = std::env::temp_dir().join(format!("soplos-live-chroot-{}.sh", std::process::id()));
    fs::write(&script_path, script).map_err(|e| e.to_string())?;
    fs::set_permissions(&script_path, fs::Permissions::from_mode(0o700)).map_err(|e| e.to_string())?;
    let script_arg = script_path.to_string_lossy().to_string();

    let terminals: Vec<(&str, Vec<String>)> = {
        let title_arg = format!("--title={title}");
        vec![
            ("ghostty", vec![title_arg.clone(), "-e".into(), "bash".into(), script_arg.clone()]),
            ("kitty", vec!["--title".into(), title.clone(), "bash".into(), script_arg.clone()]),
            ("xfce4-terminal", vec![title_arg, "-e".into(), format!("bash {script_arg}")]),
            ("konsole", vec!["--title".into(), title.clone(), "-e".into(), format!("bash {script_arg}")]),
            ("gnome-terminal", vec!["--".into(), "bash".into(), "-c".into(), script_arg.clone()]),
            ("ptyxis", vec!["--".into(), "bash".into(), script_arg.clone()]),
            ("alacritty", vec!["-e".into(), "bash".into(), script_arg.clone()]),
            ("xterm", vec!["-title".into(), title.clone(), "-e".into(), "bash".into(), script_arg.clone()]),
            ("uterm", vec!["-e".into(), format!("bash {script_arg}")]),
        ]
    };

    // The terminal of each desktop first, as in 2.x.
    let order: &[&str] = match desktop_environment().as_str() {
        "xfce" => &["ghostty", "kitty", "xfce4-terminal", "xterm", "uterm", "alacritty", "gnome-terminal"],
        "kde" => &["ghostty", "konsole", "alacritty", "xterm", "uterm", "kitty"],
        "gnome" => &["ghostty", "gnome-terminal", "ptyxis", "alacritty", "xterm", "uterm"],
        _ => &["ghostty", "kitty", "xfce4-terminal", "konsole", "gnome-terminal", "ptyxis", "xterm"],
    };

    for name in order {
        let Some((_, args)) = terminals.iter().find(|(n, _)| n == name) else { continue };
        let exists = std::env::var_os("PATH")
            .map(|paths| std::env::split_paths(&paths).any(|dir| dir.join(name).is_file()))
            .unwrap_or(false);
        if !exists {
            continue;
        }
        let spawned = Command::new(name)
            .args(args)
            .env("NO_AT_BRIDGE", "1")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
        if spawned.is_ok() {
            return Ok(());
        }
    }
    Err("no-terminal".to_string())
}
