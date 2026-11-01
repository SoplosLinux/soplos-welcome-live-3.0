import { derived, get, writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export type Theme = "light" | "dark";
/** What the user chose: a theme, or "auto" to follow the desktop. */
type ThemeChoice = "auto" | Theme;

const STORAGE_KEY = "soplos-welcome-live-theme-choice";
// How often the desktop is asked again while the window follows it.
const FOLLOW_EVERY_MS = 4000;

// The saved choice is read before the store exists, so the first
// subscription call cannot overwrite it with the default.
function readStoredChoice(): ThemeChoice {
  try {
    const value = localStorage.getItem(STORAGE_KEY);
    return value === "light" || value === "dark" ? value : "auto";
  } catch {
    return "auto";
  }
}

const choice = writable<ThemeChoice>(readStoredChoice());
choice.subscribe((value) => {
  try {
    localStorage.setItem(STORAGE_KEY, value);
  } catch {
    // Private/blocked storage: the choice still applies for this session.
  }
});

// The theme of the desktop, until it is known the dark one the app always had.
const system = writable<Theme>("dark");

/** The theme in use: the chosen one, or the desktop's while nothing is chosen. */
export const theme = derived([choice, system], ([chosen, desktop]): Theme => (chosen === "auto" ? desktop : chosen));
theme.subscribe((value) => document.documentElement.classList.toggle("light", value === "light"));

async function readSystemTheme() {
  try {
    const found = await invoke<string | null>("get_system_theme");
    if (found === "light" || found === "dark") system.set(found);
  } catch {
    // No answer: the last known theme stays.
  }
}

// The window follows the desktop from the start: it is asked now, every few
// seconds and when the window gets the focus again.
void readSystemTheme();
setInterval(() => {
  if (get(choice) === "auto") void readSystemTheme();
}, FOLLOW_EVERY_MS);
window.addEventListener("focus", () => get(choice) === "auto" && void readSystemTheme());

/**
 * The button of the window: it switches to the other theme. Switching to the
 * theme the desktop has goes back to following it, so the button needs no
 * third state.
 */
export function toggleTheme() {
  const next: Theme = get(theme) === "dark" ? "light" : "dark";
  choice.set(next === get(system) ? "auto" : next);
}
