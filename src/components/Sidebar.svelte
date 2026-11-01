<script lang="ts">
  import type { ComponentType } from "svelte";
  import { onMount, getContext } from "svelte";
  import { api } from "../lib/api";
  import { theme, toggleTheme } from "../lib/theme";
  import { Sun, Moon, Globe } from "lucide-svelte";
  import { _, locale } from "svelte-i18n";
  import { SUPPORTED_LOCALES, LOCALE_NAMES, setLocale, type SupportedLocale } from "../i18n";

  function onLanguageChange(e: Event) {
    setLocale((e.currentTarget as HTMLSelectElement).value as SupportedLocale);
  }

  export let tabs: { id: string; icon: ComponentType; label: string }[];
  export let active: string;
  export let onSelect: (id: string) => void;
  export let appVersion: string = "";
  export let envInfo: { desktopEnvironment: string; displayProtocol: string } | null = null;

  const openAbout = getContext<() => void>("openAbout");

  const DE_LABELS: Record<string, string> = {
    gnome: "GNOME",
    kde: "Plasma",
    xfce: "Xfce",
    unknown: "",
  };
  const PROTO_LABELS: Record<string, string> = {
    wayland: "Wayland",
    x11: "X11",
    unknown: "",
  };

  $: deLabel = envInfo ? (DE_LABELS[envInfo.desktopEnvironment] ?? envInfo.desktopEnvironment) : "";
  $: protoLabel = envInfo ? (PROTO_LABELS[envInfo.displayProtocol] ?? "") : "";
  $: envLabel = [deLabel, protoLabel].filter(Boolean).join(" · ");
  // "3.0.0-1-beta" -> "3.0.0-beta": the Debian revision in the middle is not shown.
  $: cleanVersion = appVersion.replace(/-\d+-/, "-");
  $: footerLabel = [envLabel, cleanVersion].filter(Boolean).join(" · ") || "Soplos Welcome Live";

  let autostartEnabled = false;

  onMount(async () => {
    try {
      autostartEnabled = await api.invoke<boolean>("get_autostart_enabled");
    } catch (error) {
      console.error("Failed to read autostart state", error);
    }
  });

  async function toggleAutostart() {
    const next = !autostartEnabled;
    try {
      await api.invoke("set_autostart_enabled", { enabled: next });
      autostartEnabled = next;
    } catch (error) {
      console.error("Failed to update autostart state", error);
    }
  }
</script>

<aside class="w-[230px] shrink-0 flex flex-col pt-[18px] px-[10px] pb-2 bg-panel border-r border-border" data-tauri-drag-region>
  <!-- With no frame of the system, the top of the sidebar moves the window. -->
  <div class="flex items-center gap-2 px-3 pb-[22px] font-bold text-[0.98rem] text-text-bright" data-tauri-drag-region="deep">
    <img src="/assets/icons/soplos-logo.png" alt="Soplos" class="w-[28px] h-[28px] object-contain shrink-0" />
    <span class="flex flex-col leading-tight min-w-0">
      <span class="whitespace-nowrap">Soplos</span>
      <span class="whitespace-nowrap">Welcome Live</span>
    </span>
  </div>

  <nav class="flex flex-col gap-[3px] flex-1">
    {#each tabs as tab}
      <button
        class="flex items-center gap-3 px-3.5 py-2.5 rounded text-[0.92rem] text-left transition-colors
          {active === tab.id
          ? 'bg-primary/15 text-primary font-semibold'
          : 'text-text-muted hover:bg-surface hover:text-text-bright'}"
        on:click={() => onSelect(tab.id)}
      >
        <svelte:component this={tab.icon} size={18} class="shrink-0" />
        <span>{tab.label}</span>
      </button>
    {/each}
  </nav>

  <div class="mt-3 flex flex-col gap-2.5">
    <div class="flex items-center justify-between gap-2 px-1">
      <span class="text-[0.78rem] text-text">{$_("sidebar.showOnStartup")}</span>
      <button
        class="relative w-[32px] h-[18px] rounded-full transition-colors shrink-0 {autostartEnabled
          ? 'bg-primary'
          : 'bg-surface-2'}"
        on:click={toggleAutostart}
        aria-pressed={autostartEnabled}
        aria-label={$_("sidebar.showOnStartup")}
      >
        <span
          class="absolute top-[2px] left-[2px] w-[14px] h-[14px] rounded-full bg-text-bright transition-transform {autostartEnabled
            ? 'translate-x-[14px]'
            : ''}"
        ></span>
      </button>
    </div>
    <div class="flex items-center gap-1.5 px-1">
      <Globe size={13} class="text-text-muted shrink-0" />
      <select
        class="appearance-none flex-1 min-w-0 bg-surface-2 border border-border rounded px-1.5 py-1 text-[0.68rem] text-text-bright outline-none focus-visible:ring-2 focus-visible:ring-primary/60"
        value={$locale}
        title={$_("sidebar.interfaceLanguage")}
        on:change={onLanguageChange}
      >
        {#each SUPPORTED_LOCALES as code}
          <option value={code}>{LOCALE_NAMES[code]}</option>
        {/each}
      </select>
    </div>
    <div class="pt-2.5 border-t border-border flex items-center gap-2 px-1">
      <button
        class="p-1 rounded text-text-muted hover:bg-surface hover:text-text-bright shrink-0"
        on:click={toggleTheme}
        title={$theme === "dark" ? $_("sidebar.switchToLight") : $_("sidebar.switchToDark")}
        aria-label={$theme === "dark" ? $_("sidebar.switchToLight") : $_("sidebar.switchToDark")}
      >
        <svelte:component this={$theme === "dark" ? Moon : Sun} size={14} />
      </button>
      <button
        class="text-[0.62rem] text-text-muted hover:text-text-bright hover:underline"
        on:click={openAbout}
        title={$_("sidebar.aboutTooltip")}
      >
        {footerLabel}
      </button>
    </div>
  </div>
</aside>
