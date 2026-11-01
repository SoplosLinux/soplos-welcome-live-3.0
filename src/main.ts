import "./styles/global.css";
import App from "./App.svelte";
import { i18nReady } from "./i18n";

// This is a desktop app, not a browser page — the WebKitGTK context menu
// (Back/Forward/Reload/Inspect) has no place here. Left enabled in dev so
// devtools stays reachable via right-click while working.
if (!import.meta.env.DEV) {
  document.addEventListener("contextmenu", (e) => e.preventDefault());
}

// svelte-i18n needs the initial locale's dictionary loaded before any
// component using $_() first renders, or those strings flash empty/as raw
// keys for a frame.
i18nReady.then(() => {
  new App({
    target: document.getElementById("app")!,
  });
});
