# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [3.0.0-2-beta] - 2026-11-14

### Changed
- Install launches Soplos Linux Installer (`/usr/bin/soplos-linux-installer`,
  as the live user) instead of Calamares: `installer.json` and the defaults
  of the code. The package recommends `soplos-linux-installer` instead of
  `calamares`, and its desktop icon follows the Desktop folder when the
  language changes, as the one of Calamares did.
- Rescue, "My computer does not start": it repairs the boot loader the
  installed system has. GRUB as before (`grub-install` and `update-grub`),
  Limine with `limine-install --force` and `limine-update`, systemd-boot with
  `bootctl install` and `kernel-install add` for every kernel.
- Rescue, "It broke after an update": the start files are rebuilt with
  `update-initramfs -u -k all` instead of `dracut --regenerate-all --force`;
  on Soplos it is dracut's own wrapper and it also runs the hooks that copy
  the new files to the EFI partition for systemd-boot. The boot entries are
  then updated for the boot loader the system has.
- NumLock (Tyron): turning it off leaves the mark
  `/etc/soplos-linux-installer/numlock-off` in the live session, which Soplos
  Linux Installer reads; the switch shows that mark. It no longer edits the
  configuration of Calamares.

### Fixed
- The EFI variables are mounted in the rescue chroot, so a repaired boot
  loader can write its entry in the firmware again.

## [3.0.0-1-beta] - 2026-11-08

### Added
- The Install page checks what the installer needs before it starts, because without it the installer stops with error 100: a real connection to the servers of Soplos (name, connection and HTTPS, and not only a connection to 8.8.8.8), and the clock of the computer against the time the servers report. A clock that is more than five minutes off shows in the list, with a button that sets it from the servers, writes it to the hardware clock and turns on the network time.
- Pressing Install with no connection, or with a wrong clock, asks first and says why the installer would fail; it can still be started.
- Regions on the Language and keyboard page, in a drop-down list: twenty for Spanish (Spain and the countries of Latin America), eight for English (United States, United Kingdom, Australia, Canada, Ireland, New Zealand, South Africa, India), France, Belgium, Switzerland and Canada for French, Germany, Austria and Switzerland for German, Italy and Switzerland for Italian, and Portugal and Brazil for Portuguese. The region sets the locale of the system (`es_MX.UTF-8`, `fr_BE.UTF-8`, ...) and its formats, and the dictionary of the language is kept.
- Keyboard layouts for Latin America (`latam`), Brazil (`br`), the United Kingdom (`gb`), Ireland (`ie`), Belgium (`be`), Switzerland (`ch`) and Canada (`ca`); each region proposes its own until the keyboard is chosen by hand.
- Help on the Rescue page, as the notes of the other 3.0 programs: a Help button over the page (or Ctrl+I) opens a column beside it with a card for each of the three options, saying what it does and what it touches.
- A banner with the image of the welcome screen, the icon and the title over every page but Home.

### Changed
- The pages have no top bar with their title any more; the strip over the content still moves the window and holds its buttons.
- The pages are no longer limited to 640 pixels of width.
- Region and keyboard are on the same row of the Language and keyboard page.

## [3.0.0-beta] - 2026-11-05

First beta. The system rescue, the password change and the repair of the initramfs and GRUB were tested in a live session of Soplos Linux Tyson (Plasma, Wayland) in a virtual machine, on an installed system with the EFI partition, `/boot` and the root partition on btrfs apart.

## [3.0.0-2-alpha] - 2026-11-05

### Fixed
- Rescue: the `/boot`, `/boot/efi`, `/efi` and `/home` partitions of the installed system were skipped without a word when the device named by their `UUID=` could not be found. The program asked `findfs` for it, and `findfs` is in `/usr/sbin`, which is not in the `PATH` of a normal user. The repairs then wrote into the empty `/boot` folder of the root partition and reported success, and GRUB found neither `/boot/grub` nor the EFI directory. The devices are now looked up in `/dev/disk/by-uuid`, `by-label`, `by-partuuid` and `by-partlabel`, with `findfs` run as the administrator as a last resort, and what cannot be mounted is written in the log.
- Rescue: reinstalling GRUB, updating GRUB and regenerating the initramfs stop with a message when the `/etc/fstab` of the installed system declares `/boot`, `/boot/efi` or `/efi` and it is not mounted, instead of repairing the wrong folder.
- Rescue: the user box of the new password form was drawn by the toolkit, white and with text that could not be read on it. It has the dark style of the rest of the window, with an arrow that shows it is a list.

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
