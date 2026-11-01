import { invoke } from "@tauri-apps/api/core";

// Thin wrapper around Tauri's invoke so views never call it directly.
// Commands are added here as they land in src-tauri/src/commands/.
export const api = {
  invoke,
};
