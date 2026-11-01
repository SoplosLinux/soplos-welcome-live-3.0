<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { _ } from "svelte-i18n";
  import { Minus, Square, Copy, X } from "lucide-svelte";

  // The buttons of the window when the system draws no frame: minimize, maximize
  // or restore, and close, at the top right of the card. They are plain icons;
  // the accent of the theme shows under the pointer, and red only on close.
  export let maximized = false;

  const win = getCurrentWindow();

  const button = "grid h-[22px] w-8 place-items-center rounded text-text-muted transition-colors";
</script>

<div class="flex items-center gap-0.5">
  <button
    class="{button} hover:bg-primary/15 hover:text-primary"
    title={$_("app.window.minimize")}
    aria-label={$_("app.window.minimize")}
    on:click={() => win.minimize()}
  >
    <Minus size={14} />
  </button>
  <button
    class="{button} hover:bg-primary/15 hover:text-primary"
    title={maximized ? $_("app.window.restore") : $_("app.window.maximize")}
    aria-label={maximized ? $_("app.window.restore") : $_("app.window.maximize")}
    on:click={() => win.toggleMaximize()}
  >
    {#if maximized}<Copy size={11} />{:else}<Square size={11} />{/if}
  </button>
  <button
    class="{button} hover:bg-danger hover:text-white"
    title={$_("app.window.close")}
    aria-label={$_("app.window.close")}
    on:click={() => win.close()}
  >
    <X size={14} />
  </button>
</div>
