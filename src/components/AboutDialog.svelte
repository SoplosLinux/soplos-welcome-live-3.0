<script lang="ts">
  import { autofocus } from "../lib/autofocus";
  import { onMount } from "svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import { open } from "@tauri-apps/plugin-shell";
  import { _ } from "svelte-i18n";
  import { fade, scale } from "svelte/transition";

  export let onClose: () => void;

  let version = "";

  onMount(async () => {
    try {
      version = await getVersion();
    } catch {
      version = "";
    }
  });
</script>

<div
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
  role="button"
  transition:fade|global={{ duration: 120 }}
  tabindex="-1"
  on:click={onClose}
  on:keydown={(e) => { if (e.key === "Escape" || (e.key === "Enter" && e.target === e.currentTarget)) onClose(); }}
>
  <div
    class="w-full max-w-[420px] bg-surface border border-border rounded-xl p-6 flex flex-col items-center gap-2 shadow-2xl text-center"
    role="presentation"
    transition:scale|global={{ duration: 140, start: 0.96 }}
    on:click|stopPropagation
  >
    <img src="/assets/icons/welcome-live.png" alt="Soplos Welcome Live" class="w-16 h-16 object-contain" />
    <h2 class="text-text-bright font-semibold text-[1.05rem] mt-1">Soplos Welcome Live</h2>
    <p class="text-[0.78rem] text-text-muted">{version ? $_("about.version", { values: { version } }) : ""}</p>
    <p class="text-[0.82rem] text-text mt-1">{$_("about.tagline")}</p>

    <div class="w-full border-t border-border my-2"></div>

    <p class="text-[0.75rem] text-text-muted">{$_("about.copyright")}</p>
    <p class="text-[0.75rem] text-text-muted">{$_("about.license")}</p>
    <button
      class="text-[0.8rem] text-primary hover:underline mt-1"
      on:click={() => open("https://soplos.org")}
    >
      soplos.org
    </button>

    <button
      class="mt-4 px-6 py-2 rounded bg-primary hover:bg-primary-hover text-text-bright text-[0.82rem] font-semibold"
      use:autofocus
          on:click={onClose}
    >
      {$_("common.close")}
    </button>
  </div>
</div>
