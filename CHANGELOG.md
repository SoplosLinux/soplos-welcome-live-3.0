# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [3.0.0-1-alpha] - 2026-11-04

### Fixed
- The application metadata points to the launcher that the package installs, `soplos-welcome-live.desktop`, so the software centers link the entry of the menu with the program.
- The manual page of the program is the same text in the project and in the package.

## [3.0.0-alpha] - 2026-10-31

### Added
- First version of Soplos Welcome Live 3.0, a port of 2.x to Tauri 2 and Svelte with the layout of Soplos Welcome 3.0: frameless window, sidebar, light and dark theme that follows the desktop, and the 8 languages of the dictionaries.
- Tabs: Home, Install, Rescue, Tools, and Language and keyboard.
- Install: the installer is read from `installer.json` (Calamares today), so replacing it means replacing that file; checks of the internet connection, UEFI, disks and power before starting.
- Rescue in plain language: the computer does not start, a forgotten password, a failure after an update. Installed systems are found automatically (BTRFS subvolumes included) and the other partitions are mounted from the system's own `/etc/fstab`. The terminal of the 2.x chroot is kept as an advanced option.
- Language and keyboard change, ported step by step from 2.x, with the keyboard layout chosen on its own for each of the 8 languages.
- Tools: GParted, hardware information, screen resolution (X11 and Plasma) and NumLock for the installed Tyron system.
- Six screenshots, the manual page, the AppStream metadata with its captions in the 8 languages and the README.

### Known gaps
- The screen resolution is not available on GNOME yet: the D-Bus route of 2.x is not ported.
