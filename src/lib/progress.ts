import { writable } from "svelte/store";

/**
 * The bar along the bottom edge of the window. `null` hides it, a number from
 * 0 to 1 fills it, and `"busy"` slides it for work whose end cannot be known
 * (launching the installer, mounting a disk).
 */
export const progress = writable<number | "busy" | null>(null);
