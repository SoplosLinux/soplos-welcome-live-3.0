// Svelte action: focuses the element as soon as it's mounted. Used on a
// popup's primary button so Enter accepts it immediately (native <button>
// behavior needs the button to actually have focus — the overlay's own
// tabindex="-1" is never focused automatically, so without this, Enter did
// nothing until the user first pressed Tab).
export function autofocus(node: HTMLElement) {
  node.focus();
}
