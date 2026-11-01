// The light or dark theme of the desktop, so the window can follow it until
// the user chooses one. Every desktop says it in its own way: the XDG portal
// answers on GNOME and Plasma, Plasma also keeps its colors in kdeglobals, and
// Xfce only has the name of the GTK theme, so there the answer is a guess from
// that name.

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const LIMIT: Duration = Duration::from_secs(2);

fn has_command(name: &str) -> bool {
    std::env::var_os("PATH").map(|path| std::env::split_paths(&path).any(|dir| dir.join(name).is_file())).unwrap_or(false)
}

/// Stdout of a command run with the C locale, stopped when it takes longer
/// than `limit`; None when it fails.
fn output_within(program: &str, args: &[&str], limit: Duration) -> Option<String> {
    use std::io::Read;
    let mut child = Command::new(program)
        .args(args)
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let mut stdout = child.stdout.take()?;
    let reader = std::thread::spawn(move || {
        let mut text = String::new();
        let _ = stdout.read_to_string(&mut text);
        text
    });
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() < limit => std::thread::sleep(Duration::from_millis(50)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    };
    let text = reader.join().ok()?;
    status.success().then_some(text)
}

/// The value the portal gives for `org.freedesktop.appearance color-scheme`
/// (1 dark, 2 light, 0 no preference), from the text `gdbus` prints, such as
/// `(<<uint32 1>>,)`.
fn parse_portal(text: &str) -> Option<&'static str> {
    let digits: String = text.rsplit("uint32").next()?.chars().filter(|c| c.is_ascii_digit()).collect();
    match digits.as_str() {
        "1" => Some("dark"),
        "2" => Some("light"),
        _ => None,
    }
}

/// `org.gnome.desktop.interface color-scheme`, as `gsettings` prints it.
fn parse_gnome(text: &str) -> Option<&'static str> {
    match text.trim().trim_matches('\'') {
        "prefer-dark" => Some("dark"),
        "prefer-light" => Some("light"),
        _ => None,
    }
}

/// Whether a theme is dark, judged by its name. It is only a guess: it is
/// what an Xfce theme gives, or a GNOME one when the scheme says nothing.
fn name_is_dark(name: &str) -> bool {
    let lower = name.to_lowercase();
    ["dark", "nokto", "black", "night", "noir"].iter().any(|word| lower.contains(word))
}

/// A color "r,g,b" of kdeglobals is dark when its brightness is low.
fn rgb_is_dark(text: &str) -> Option<bool> {
    let parts: Vec<f64> = text.trim().split(',').filter_map(|p| p.trim().parse::<f64>().ok()).collect();
    if parts.len() < 3 {
        return None;
    }
    let luminance = 0.2126 * parts[0] + 0.7152 * parts[1] + 0.0722 * parts[2];
    Some(luminance < 128.0)
}

fn portal() -> Option<&'static str> {
    let text = output_within(
        "gdbus",
        &[
            "call",
            "--session",
            "--dest",
            "org.freedesktop.portal.Desktop",
            "--object-path",
            "/org/freedesktop/portal/desktop",
            "--method",
            "org.freedesktop.portal.Settings.Read",
            "org.freedesktop.appearance",
            "color-scheme",
        ],
        LIMIT,
    )?;
    parse_portal(&text)
}

fn gnome() -> Option<&'static str> {
    let scheme = output_within("gsettings", &["get", "org.gnome.desktop.interface", "color-scheme"], LIMIT)?;
    parse_gnome(&scheme)
}

fn gtk_theme_name() -> Option<String> {
    let name = output_within("gsettings", &["get", "org.gnome.desktop.interface", "gtk-theme"], LIMIT)?;
    Some(name.trim().trim_matches('\'').to_string()).filter(|n| !n.is_empty())
}

fn plasma() -> Option<&'static str> {
    for tool in ["kreadconfig6", "kreadconfig5"] {
        if !has_command(tool) {
            continue;
        }
        let color = output_within(tool, &["--file", "kdeglobals", "--group", "Colors:Window", "--key", "BackgroundNormal"], LIMIT)?;
        if let Some(dark) = rgb_is_dark(&color) {
            return Some(if dark { "dark" } else { "light" });
        }
    }
    None
}

fn xfce() -> Option<&'static str> {
    let name = output_within("xfconf-query", &["-c", "xsettings", "-p", "/Net/ThemeName"], LIMIT)?;
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    Some(if name_is_dark(name) { "dark" } else { "light" })
}

/// "dark" or "light" as the desktop has it, None when it cannot be told.
pub fn detect() -> Option<&'static str> {
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default().to_lowercase();
    if desktop.contains("xfce") {
        return xfce().or_else(portal);
    }
    if desktop.contains("kde") || desktop.contains("plasma") {
        return portal().or_else(plasma);
    }
    portal().or_else(gnome).or_else(|| gtk_theme_name().map(|n| if name_is_dark(&n) { "dark" } else { "light" }))
}

/// The theme of the desktop, "dark" or "light"; None when it cannot be told.
#[tauri::command]
pub async fn get_system_theme() -> Option<String> {
    tauri::async_runtime::spawn_blocking(|| detect().map(String::from)).await.ok().flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_portal_answer_is_dark_light_or_nothing() {
        assert_eq!(parse_portal("(<<uint32 1>>,)"), Some("dark"));
        assert_eq!(parse_portal("(<<uint32 2>>,)\n"), Some("light"));
        assert_eq!(parse_portal("(<<uint32 0>>,)"), None);
        assert_eq!(parse_portal(""), None);
    }

    #[test]
    fn gnome_says_its_scheme_in_words() {
        assert_eq!(parse_gnome("'prefer-dark'\n"), Some("dark"));
        assert_eq!(parse_gnome("'prefer-light'"), Some("light"));
        assert_eq!(parse_gnome("'default'"), None);
    }

    #[test]
    fn a_dark_theme_is_told_by_its_name() {
        for name in ["Adwaita-dark", "Arc-Dark", "Nordic-darker", "Mint-Y-Dark-Aqua", "Materia-dark-compact", "Nokto"] {
            assert!(name_is_dark(name), "{name}");
        }
        for name in ["Adwaita", "Arc", "Greybird", "Mint-Y", "Breeze"] {
            assert!(!name_is_dark(name), "{name}");
        }
    }

    #[test]
    fn plasma_colors_are_told_by_their_brightness() {
        assert_eq!(rgb_is_dark("239,240,241"), Some(false));
        assert_eq!(rgb_is_dark("35,38,41"), Some(true));
        assert_eq!(rgb_is_dark("nothing"), None);
    }
}
