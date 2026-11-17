<script lang="ts">
  import { setContext, onMount, type ComponentType } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { getVersion } from "@tauri-apps/api/app";
  import { _ } from "svelte-i18n";
  import { api } from "./lib/api";
  import { progress } from "./lib/progress";
  import Sidebar from "./components/Sidebar.svelte";
  import WindowControls from "./components/WindowControls.svelte";
  import AboutDialog from "./components/AboutDialog.svelte";
  import RescueNotes from "./components/RescueNotes.svelte";
  import PageHero from "./components/PageHero.svelte";
  import { modalDepth, showNotes } from "./lib/ui";
  import { Eye } from "lucide-svelte";
  import Home from "./views/Home.svelte";
  import Install from "./views/Install.svelte";
  import Rescue from "./views/Rescue.svelte";
  import Tools from "./views/Tools.svelte";
  import Language from "./views/Language.svelte";
  import { Home as HomeIcon, Download, Wrench, LayoutGrid, Keyboard } from "lucide-svelte";

  $: tabs = [
    { id: "home", icon: HomeIcon, label: $_("app.tabs.home.label"), title: $_("app.tabs.home.title") },
    { id: "language", icon: Keyboard, label: $_("app.tabs.language.label"), title: $_("app.tabs.language.title") },
    { id: "install", icon: Download, label: $_("app.tabs.install.label"), title: $_("app.tabs.install.title") },
    { id: "rescue", icon: Wrench, label: $_("app.tabs.rescue.label"), title: $_("app.tabs.rescue.title") },
    { id: "tools", icon: LayoutGrid, label: $_("app.tabs.tools.label"), title: $_("app.tabs.tools.title") },
  ];

  const views: Record<string, ComponentType> = {
    home: Home,
    install: Install,
    rescue: Rescue,
    tools: Tools,
    language: Language,
  };

  let active = "home";
  let appVersion = "";
  let envInfo: { desktopEnvironment: string; displayProtocol: string; edition: string } | null = null;

  // With `decorations` off in tauri.conf.json (and `transparent` on) the system
  // draws no frame: the window is one card with rounded corners and the buttons
  // of the window float at its top right. With `decorations` on, nothing of
  // this shows, so that setting is the only switch between the two.
  let frameless = false;
  let maximized = false;

  onMount(() => {
    let unlisten: (() => void) | undefined;
    (async () => {
      try {
        appVersion = await getVersion();
      } catch {
        appVersion = "";
      }
      try {
        envInfo = await api.invoke("get_environment_info");
      } catch {
        envInfo = null;
      }
      try {
        const win = getCurrentWindow();
        frameless = !(await win.isDecorated());
        if (frameless) {
          maximized = await win.isMaximized();
          unlisten = await win.onResized(async () => {
            maximized = await win.isMaximized();
          });
        }
      } catch {
        frameless = false;
      }
    })();
    return () => unlisten?.();
  });

  function select(id: string) {
    active = id;
    document.getElementById("main-scroll")?.scrollTo({ top: 0 });
  }

  // Lets any view (e.g. Home's cards) switch tabs without prop-drilling.
  setContext("navigate", select);
  setContext("openAbout", () => (aboutOpen = true));
  setContext("environment", () => envInfo);

  $: ActiveView = views[active];
  $: activeTab = tabs.find((t) => t.id === active)!;

  // The pages that have notes beside them.
  $: notesAvailable = active === "rescue";

  // F1 -> About, F5 -> reload the page, Ctrl+I -> notes, Ctrl+Q -> quit,
  // Ctrl+Tab / Ctrl+Shift+Tab -> next or previous page. Quiet while a dialog
  // is open (it closes itself with Esc/Enter).
  let aboutOpen = false;
  // Bumped by F5: the page on screen is built again and reads its data anew.
  let reloadKey = 0;

  function handleKeydown(e: KeyboardEvent) {
    if (aboutOpen || $modalDepth > 0) return;
    if (e.key === "F1") {
      e.preventDefault();
      aboutOpen = true;
      return;
    }
    if (e.key === "F5") {
      e.preventDefault();
      // Never in the middle of a job (an installer check, a repair, a language change).
      if ($progress === null) reloadKey += 1;
      return;
    }
    if (!e.ctrlKey) return;
    if (e.key === "i" || e.key === "I") {
      e.preventDefault();
      if (notesAvailable) showNotes.update((v) => !v);
    } else if (e.key === "q" || e.key === "Q") {
      e.preventDefault();
      getCurrentWindow().close();
    } else if (e.code === "Tab") {
      e.preventDefault();
      const i = tabs.findIndex((t) => t.id === active);
      select(tabs[(i + (e.shiftKey ? -1 : 1) + tabs.length) % tabs.length].id);
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="relative flex h-screen overflow-hidden bg-dark {frameless && !maximized ? 'window-card' : ''}">
  <Sidebar {tabs} {active} onSelect={select} {appVersion} {envInfo} />

  {#if frameless}
    <div class="absolute right-2 top-[5px] z-40"><WindowControls {maximized} /></div>
  {/if}


  <div class="flex-1 flex flex-col overflow-hidden min-w-0">
    <!-- No page has a top bar: the strip over the content holds the window
         buttons and moves the window. -->
    {#if frameless || notesAvailable}
      <div class="h-7 shrink-0 flex items-center justify-end gap-2 {frameless ? 'pr-[116px]' : 'pr-7'}" data-tauri-drag-region>
        {#if notesAvailable}
          <button
            class="flex items-center gap-1.5 rounded-md border bg-surface px-[9px] py-0.5 text-[0.78rem] hover:bg-surface-2 {$showNotes ? 'border-primary text-primary' : 'border-border text-text'}"
            aria-pressed={$showNotes}
            title="{$_('sidebar.notes')} (Ctrl+I)"
            on:click={() => showNotes.update((v) => !v)}
          >
            <Eye size={13} />
            {$_("sidebar.notes")}
          </button>
        {/if}
      </div>
    {/if}
    <div id="main-scroll" class="content-scroll flex-1 overflow-y-auto p-7 {frameless || notesAvailable ? 'pt-0' : ''}">
      {#if active !== "home"}
        <PageHero title={activeTab.title} icon={activeTab.icon} />
      {/if}
      {#key reloadKey}
        {#if notesAvailable && $showNotes}
          <div class="grid gap-4 items-start min-[900px]:grid-cols-[minmax(0,1fr)_290px]">
            <svelte:component this={ActiveView} />
            <RescueNotes />
          </div>
        {:else}
          <svelte:component this={ActiveView} />
        {/if}
      {/key}
    </div>

    <div class="h-1.5 bg-surface-2 overflow-hidden shrink-0" class:invisible={$progress === null}>
      {#if typeof $progress === "number"}
        <div class="h-full bg-primary transition-[width]" style="width: {$progress * 100}%"></div>
      {:else if $progress === "busy"}
        <div class="h-full w-1/3 bg-primary progress-fill"></div>
      {/if}
    </div>
  </div>

  {#if aboutOpen}
    <AboutDialog onClose={() => (aboutOpen = false)} />
  {/if}
</div>
