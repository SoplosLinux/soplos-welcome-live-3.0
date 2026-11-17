import { writable } from "svelte/store";

// Whether the explanatory notes are shown: a panel beside the page that has
// them. Off on every launch; the user turns them on with the Notes switch of
// the sidebar (or Ctrl+I).
export const showNotes = writable<boolean>(false);

// Dialogs open right now. The keyboard shortcuts stay quiet while one is
// open, so nothing changes page or quits behind a confirmation.
export const modalDepth = writable(0);
