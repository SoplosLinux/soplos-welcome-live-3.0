// Language and keyboard change of the live session, ported step by step from
// 2.x's utils/language_changer.py (and utils/update_gtk_bookmarks.py). The
// frontend calls the steps in order so the progress bar shows real work:
//   locale -> user -> xdg -> bookmarks -> restart
// The keyboard layout is a separate choice now; 2.x derived it from the
// language.

use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use super::environment::desktop_environment;

const LOCALES: [(&str, &str); 8] = [
    ("es", "es_ES.UTF-8"),
    ("en", "en_US.UTF-8"),
    ("fr", "fr_FR.UTF-8"),
    ("de", "de_DE.UTF-8"),
    ("pt", "pt_PT.UTF-8"),
    ("it", "it_IT.UTF-8"),
    ("ro", "ro_RO.UTF-8"),
    ("ru", "ru_RU.UTF-8"),
];

/// The keyboard layouts offered, one per language of the dictionaries.
const LAYOUTS: [&str; 8] = ["es", "us", "fr", "de", "pt", "it", "ro", "ru"];

#[derive(Serialize)]
pub struct LanguageState {
    pub language: String,
    pub layout: String,
}

fn home() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_default())
}

fn username() -> String {
    std::env::var("USER").unwrap_or_else(|_| "liveuser".to_string())
}

fn locale_for(lang: &str) -> Result<&'static str, String> {
    LOCALES
        .iter()
        .find(|(code, _)| *code == lang)
        .map(|(_, locale)| *locale)
        .ok_or_else(|| format!("unsupported language: {lang}"))
}

fn check_layout(layout: &str) -> Result<(), String> {
    if LAYOUTS.contains(&layout) {
        Ok(())
    } else {
        Err(format!("unsupported layout: {layout}"))
    }
}

#[tauri::command]
pub fn get_language_state() -> LanguageState {
    let lang = std::env::var("LANG").unwrap_or_else(|_| "en_US.UTF-8".to_string());
    let code = lang.split('_').next().unwrap_or("en").to_string();
    let code = if LOCALES.iter().any(|(c, _)| *c == code) { code } else { "en".to_string() };
    let layout = match code.as_str() {
        "en" => "us".to_string(),
        other => other.to_string(),
    };
    LanguageState { language: code, layout }
}

#[tauri::command(async)]
pub fn language_step(step: String, lang: String, layout: String) -> Result<(), String> {
    let locale = locale_for(&lang)?;
    check_layout(&layout)?;
    match step.as_str() {
        "locale" => configure_system_locale(locale, &layout),
        "user" => {
            apply_user_settings(&lang, locale, &layout);
            Ok(())
        }
        "xdg" => {
            migrate_xdg_directories(locale);
            Ok(())
        }
        "bookmarks" => {
            if matches!(desktop_environment().as_str(), "gnome" | "xfce") {
                update_gtk_bookmarks();
            }
            Ok(())
        }
        "restart" => restart_display_manager(),
        other => Err(format!("unknown step: {other}")),
    }
}

/// All the changes that need root, in one script and one sudo call.
fn configure_system_locale(locale: &str, layout: &str) -> Result<(), String> {
    let user = username();
    let short = locale.split('_').next().unwrap_or("en");
    let bare = locale.replace(".UTF-8", "");

    let dm_config = match desktop_environment().as_str() {
        "kde" => format!(
            r#"
# SDDM configuration
if [ -d "/etc/sddm.conf.d" ]; then
    echo "[X11]" > /etc/sddm.conf.d/keyboard.conf
    echo "ServerArguments=-layout {layout}" >> /etc/sddm.conf.d/keyboard.conf
    echo "[General]" > /etc/sddm.conf.d/locale.conf
    echo "Language={locale}" >> /etc/sddm.conf.d/locale.conf
fi
"#
        ),
        "gnome" => format!(
            r#"
# GDM/AccountsService configuration
if [ -f /var/lib/AccountsService/users/{user} ]; then
    if grep -q "^Language=" /var/lib/AccountsService/users/{user}; then
        sed -i "s/^Language=.*/Language={locale}/" /var/lib/AccountsService/users/{user}
    else
        echo "Language={locale}" >> /var/lib/AccountsService/users/{user}
    fi
fi
"#
        ),
        "xfce" => format!(
            r#"
# LightDM configuration
if [ -f /etc/lightdm/lightdm.conf ]; then
    sed -i "s/^#*greeter-locale=.*/greeter-locale={locale}/" /etc/lightdm/lightdm.conf 2>/dev/null || true
fi
"#
        ),
        _ => String::new(),
    };

    let script = format!(
        r#"#!/bin/bash
# System locale configuration, all in one script

# 1. /etc/locale.conf
cat > /etc/locale.conf << EOF
LANG={locale}
LC_ADDRESS={locale}
LC_IDENTIFICATION={locale}
LC_MEASUREMENT={locale}
LC_MONETARY={locale}
LC_NAME={locale}
LC_NUMERIC={locale}
LC_PAPER={locale}
LC_TELEPHONE={locale}
LC_TIME={locale}
EOF

# 2. /etc/default/locale
cat > /etc/default/locale << EOF
LANG={locale}
LANGUAGE={short}
LC_ALL={locale}
EOF

# 3. /etc/environment
cat > /etc/environment << EOF
LANG={locale}
LANGUAGE={short}
EOF

# 4. Generate the locale only when it is missing
if ! locale -a | grep -i "{bare}" > /dev/null; then
    locale-gen {locale} 2>/dev/null || true
fi

# 5. localectl
localectl set-locale LANG={locale} 2>/dev/null || true
localectl set-x11-keymap {layout} 2>/dev/null || true

# 6. Wayland keyboard config
mkdir -p /etc/xdg
echo "XKBLAYOUT={layout}" > /etc/xdg/keyboard

# 7. Console keyboard
echo "KEYMAP={layout}" > /etc/vconsole.conf 2>/dev/null || true

# 8. X11 keyboard config
mkdir -p /etc/X11/xorg.conf.d
cat > /etc/X11/xorg.conf.d/00-keyboard.conf << EOF
Section "InputClass"
    Identifier "system-keyboard"
    MatchIsKeyboard "on"
    Option "XkbLayout" "{layout}"
EndSection
EOF

{dm_config}

echo "System locale configured"
"#
    );

    // The script goes to sudo's stdin: no file in /tmp for anyone to swap.
    let mut child = Command::new("sudo")
        .args(["-n", "/bin/bash", "-s"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    child
        .stdin
        .take()
        .ok_or_else(|| "no stdin".to_string())?
        .write_all(script.as_bytes())
        .map_err(|e| e.to_string())?;
    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn run_quiet(program: &str, args: &[&str]) {
    let _ = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

fn kwriteconfig() -> &'static str {
    if which("kwriteconfig6") {
        "kwriteconfig6"
    } else {
        "kwriteconfig5"
    }
}

fn which(program: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|dir| dir.join(program).is_file()))
        .unwrap_or(false)
}

/// User-level settings, no sudo.
fn apply_user_settings(lang: &str, locale: &str, layout: &str) {
    std::env::set_var("LANG", locale);
    std::env::set_var("LC_ALL", locale);
    std::env::set_var("LANGUAGE", lang);

    match desktop_environment().as_str() {
        "kde" => {
            let kw = kwriteconfig();
            run_quiet(kw, &["--file", "plasma-localerc", "--group", "Translations", "--key", "LANGUAGE", lang]);
            run_quiet(kw, &["--file", "plasma-localerc", "--group", "Formats", "--key", "LANG", locale]);
            run_quiet(kw, &["--file", "kxkbrc", "--group", "Layout", "--key", "LayoutList", layout]);
            run_quiet(kw, &["--file", "kxkbrc", "--group", "Layout", "--key", "Use", "true"]);
            run_quiet("qdbus6", &["org.kde.keyboard", "/Layouts", "reset"]);
        }
        "gnome" => {
            run_quiet("gsettings", &["set", "org.gnome.system.locale", "region", locale]);
            run_quiet(
                "gsettings",
                &["set", "org.gnome.desktop.input-sources", "sources", &format!("[('xkb', '{layout}')]")],
            );
        }
        "xfce" => {
            run_quiet("xfconf-query", &["-c", "keyboard-layout", "-p", "/Default/XkbLayout", "-s", layout]);
        }
        _ => {}
    }
    // Works on X11.
    run_quiet("setxkbmap", &[layout]);
}

fn parse_user_dirs(file: &Path) -> Vec<(String, PathBuf)> {
    let home = home();
    let mut dirs = Vec::new();
    let Ok(content) = fs::read_to_string(file) else { return dirs };
    for line in content.lines() {
        let line = line.trim();
        if !line.starts_with("XDG_") {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else { continue };
        let value = value.trim_matches('"');
        let path = if let Some(rest) = value.strip_prefix("$HOME/") {
            home.join(rest)
        } else if value.starts_with('/') {
            PathBuf::from(value)
        } else {
            home.join(value)
        };
        dirs.push((key.to_string(), path));
    }
    dirs
}

fn lookup(dirs: &[(String, PathBuf)], key: &str) -> Option<PathBuf> {
    dirs.iter().find(|(k, _)| k == key).map(|(_, p)| p.clone())
}

/// Moves what the old folders hold to the new ones. When the same name exists
/// in the destination the source is dropped, so the old folder can be removed.
fn migrate_xdg_directories(locale: &str) {
    let home = home();
    let user_dirs_file = home.join(".config").join("user-dirs.dirs");
    let old_dirs = parse_user_dirs(&user_dirs_file);

    let short = locale.split('_').next().unwrap_or("en");
    let _ = Command::new("xdg-user-dirs-update")
        .arg("--force")
        .env("LANG", locale)
        .env("LC_ALL", locale)
        .env("LANGUAGE", short)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    let new_dirs = parse_user_dirs(&user_dirs_file);
    let desktop = desktop_environment();

    for (key, old_path) in &old_dirs {
        let Some(new_path) = lookup(&new_dirs, key) else { continue };
        if &new_path == old_path || !old_path.exists() {
            continue;
        }
        let _ = fs::create_dir_all(&new_path);
        if new_path.exists() {
            if let Ok(entries) = fs::read_dir(old_path) {
                for entry in entries.flatten() {
                    let item = entry.path();
                    let dest = new_path.join(entry.file_name());
                    if dest.exists() {
                        if item.is_dir() {
                            let _ = fs::remove_dir_all(&item);
                        } else {
                            let _ = fs::remove_file(&item);
                        }
                    } else {
                        let _ = fs::rename(&item, &dest);
                    }
                }
            }
        }

        if key == "XDG_DESKTOP_DIR" {
            for name in ["calamares-install-soplos.desktop", "home.desktop", "trash.desktop"] {
                let old_file = old_path.join(name);
                let new_file = new_path.join(name);
                if old_file.exists() && !new_file.exists() {
                    let _ = fs::rename(&old_file, &new_file);
                }
            }
            if desktop == "gnome" {
                restore_gnome_trust(&new_path);
            }
        }

        // Remove the old folder only when it ended up empty.
        let empty = fs::read_dir(old_path).map(|mut d| d.next().is_none()).unwrap_or(false);
        if empty {
            let _ = fs::remove_dir(old_path);
        }
    }

    if desktop == "kde" {
        update_kde_references(&new_dirs);
    }
}

/// The moved launchers need their execute bit and the GNOME trust flag again.
fn restore_gnome_trust(dir: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
            continue;
        }
        if let Ok(meta) = fs::metadata(&path) {
            let mut perms = meta.permissions();
            perms.set_mode(perms.mode() | 0o111);
            let _ = fs::set_permissions(&path, perms);
        }
        run_quiet("gio", &["set", &path.to_string_lossy(), "metadata::trusted", "true"]);
    }
}

fn update_kde_references(new_dirs: &[(String, PathBuf)]) {
    let home = home();
    let kw = kwriteconfig();
    run_quiet(
        kw,
        &["--file", "dolphinrc", "--group", "General", "--key", "HomeUrl", &format!("file://{}", home.display())],
    );
    let desktop_path = lookup(new_dirs, "XDG_DESKTOP_DIR").unwrap_or_else(|| home.join("Desktop"));
    run_quiet(
        kw,
        &[
            "--file",
            "plasma-org.kde.plasma.desktop-appletsrc",
            "--group",
            "Containments",
            "--group",
            "1",
            "--group",
            "General",
            "--key",
            "url",
            &format!("file://{}", desktop_path.display()),
        ],
    );
    // Dolphin regenerates its places from the new folders.
    let _ = fs::remove_file(home.join(".local/share/user-places.xbel"));
    let _ = fs::remove_file(home.join(".local/share/Trash/files/user-places.xbel"));
    run_quiet(
        "qdbus6",
        &["org.kde.plasmashell", "/PlasmaShell", "org.kde.PlasmaShell.evaluateScript", "refreshCurrentShell()"],
    );
}

/// Points the GTK bookmarks (Nautilus, Thunar) at the new XDG folders and
/// keeps the custom ones the user had.
fn update_gtk_bookmarks() {
    let home = home();
    let xdg = parse_user_dirs(&home.join(".config").join("user-dirs.dirs"));
    if xdg.is_empty() {
        return;
    }
    let order = ["XDG_DOCUMENTS_DIR", "XDG_DOWNLOAD_DIR", "XDG_MUSIC_DIR", "XDG_PICTURES_DIR", "XDG_VIDEOS_DIR"];
    let mut fresh: Vec<String> = Vec::new();
    for key in order {
        if let Some(path) = lookup(&xdg, key) {
            let _ = fs::create_dir_all(&path);
            fresh.push(format!("file://{}", path.display()));
        }
    }
    let english = ["Documents", "Downloads", "Music", "Pictures", "Videos", "Desktop", "Public", "Templates"];

    for target in [home.join(".config/gtk-3.0/bookmarks"), home.join(".config/gtk-4.0/bookmarks")] {
        if let Some(parent) = target.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let mut custom: Vec<String> = Vec::new();
        if let Ok(content) = fs::read_to_string(&target) {
            for line in content.lines().map(str::trim).filter(|l| !l.is_empty()) {
                let is_xdg = xdg.iter().any(|(_, p)| line.contains(&format!("file://{}", p.display())));
                if is_xdg {
                    continue;
                }
                let uri = line.split_whitespace().next().unwrap_or("");
                let Some(local) = uri.strip_prefix("file://") else { continue };
                let path = PathBuf::from(local);
                let legacy = path.parent() == Some(home.as_path())
                    && path.file_name().and_then(|n| n.to_str()).map(|n| english.contains(&n)).unwrap_or(false);
                if path.exists() && !legacy && path != home {
                    custom.push(line.to_string());
                }
            }
        }
        let mut all = fresh.clone();
        all.extend(custom);
        let _ = fs::write(&target, all.join("\n") + "\n");
    }

    run_quiet("nautilus", &["-q"]);
    if Command::new("pgrep").arg("Thunar").stdout(Stdio::null()).status().map(|s| s.success()).unwrap_or(false) {
        run_quiet("thunar", &["-q"]);
    }
}

fn active_display_manager() -> String {
    let candidates: &[&str] = match desktop_environment().as_str() {
        "kde" => &["plasmalogin", "sddm"],
        "xfce" => &["lightdm"],
        "gnome" => &["gdm3", "gdm"],
        _ => &[],
    };
    for dm in candidates {
        let output = Command::new("systemctl").args(["is-active", dm]).output();
        if let Ok(output) = output {
            if String::from_utf8_lossy(&output.stdout).trim() == "active" {
                return dm.to_string();
            }
        }
    }
    "display-manager.service".to_string()
}

fn restart_display_manager() -> Result<(), String> {
    run_quiet("sync", &[]);
    std::thread::sleep(Duration::from_secs(1));

    if desktop_environment() == "gnome" {
        // 'systemctl restart gdm3' freezes the live session on GNOME: the
        // session is killed instead and GDM's autologin starts the new one.
        let user = username();
        let spawned = Command::new("sh").arg("-c").arg(format!("pkill -KILL -u {user}")).spawn();
        if spawned.is_err() {
            run_quiet("gnome-session-quit", &["--force", "--no-prompt"]);
        }
        std::process::exit(0);
    }

    let dm = active_display_manager();
    let status = Command::new("sudo")
        .args(["-n", "systemctl", "restart", &dm])
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("could not restart {dm}"))
    }
}
