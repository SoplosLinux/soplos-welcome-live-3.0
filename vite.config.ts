import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// Tauri expects a fixed port and ignores the src-tauri directory for HMR
export default defineConfig({
  plugins: [svelte()],
  // The page is loaded from the disk by a desktop window, so the size of a
  // chunk does not matter the way it does on the web: no warning about it.
  build: {
    chunkSizeWarningLimit: 2000,
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});
