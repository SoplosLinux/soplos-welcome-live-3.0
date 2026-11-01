<script lang="ts">
  import { onMount, getContext } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { open } from "@tauri-apps/plugin-shell";
  import { _ } from "svelte-i18n";
  import SystemCard from "../components/SystemCard.svelte";
  import AppCard from "../components/AppCard.svelte";
  import { api } from "../lib/api";
  import {
    Cpu,
    MemoryStick,
    CircuitBoard,
    HardDrive,
    Download,
    Wrench,
    LayoutGrid,
    Globe,
    MessageCircle,
    BookOpen,
    Heart,
  } from "lucide-svelte";

  const navigate = getContext<(id: string) => void>("navigate");
  const environment = getContext<() => { edition: string; desktopEnvironment: string } | null>("environment");

  interface SystemInfo {
    cpu: string;
    memory: string;
    gpu: string;
    storage: string;
    uefi: boolean;
  }

  let info: SystemInfo | null = null;
  let edition = "unknown";

  onMount(async () => {
    try {
      info = await api.invoke<SystemInfo>("get_system_info");
    } catch (error) {
      console.error("Failed to read system info", error);
    }
    try {
      const env = await api.invoke<{ edition: string }>("get_environment_info");
      edition = env.edition;
    } catch {
      edition = environment()?.edition ?? "unknown";
    }
  });

  $: system = [
    { icon: Cpu, label: $_("home.system.cpu"), value: info?.cpu || "—" },
    { icon: MemoryStick, label: $_("home.system.memory"), value: info?.memory || "—" },
    { icon: CircuitBoard, label: $_("home.system.graphics"), value: info?.gpu || "—" },
    {
      icon: HardDrive,
      label: $_("home.system.storage"),
      value: info?.storage ? `${info.storage} · ${info.uefi ? "UEFI" : "BIOS"}` : "—",
    },
  ];

  $: links = [
    { label: $_("home.links.web"), icon: Globe, url: "https://soplos.org" },
    { label: $_("home.links.forums"), icon: MessageCircle, url: "https://soplos.org/forums/" },
    { label: $_("home.links.wiki"), icon: BookOpen, url: "https://soplos.org/wiki" },
    { label: $_("home.links.donate"), icon: Heart, url: "https://soplos.org/community/donate/" },
  ];

  $: editionLabel = edition === "unknown" ? "" : $_(`home.editions.${edition}`);
</script>

<div class="flex flex-col min-h-full">
  <div
    class="rounded px-5 py-4 mb-3.5 flex flex-col items-center text-center border border-border"
    style="background: linear-gradient(rgba(20, 20, 22, 0.55), rgba(20, 20, 22, 0.82)),
      url('/assets/hero/welcome-live-background.jpg'); background-size: cover; background-position: center;"
  >
    <img src="/assets/icons/soplos-logo.png" alt="Soplos" class="w-12 h-12 object-contain mb-1.5" />
    <h2 class="text-[1.4rem] font-bold text-white mb-0.5">{$_("home.title")}</h2>
    {#if editionLabel}
      <p class="text-[0.85rem] mb-1.5 font-semibold" style="color: #ffb27a;">{editionLabel}</p>
    {/if}
    <p class="text-white/85 text-[0.8rem] leading-snug max-w-[540px] mb-3.5">
      {$_("home.description")}
    </p>
    <div class="flex flex-wrap items-center justify-center gap-2.5">
      <button
        class="bg-primary hover:bg-primary-hover text-white px-5 py-2 rounded font-semibold text-[0.86rem]"
        on:click={() => navigate("install")}
      >
        {$_("home.install")}
      </button>
      <button
        class="bg-white/15 hover:bg-white/25 border border-white/35 text-white px-5 py-2 rounded font-semibold text-[0.86rem]"
        on:click={() => getCurrentWindow().close()}
      >
        {$_("home.keepTrying")}
      </button>
    </div>
  </div>

  <div class="grid grid-cols-2 gap-3.5 mb-3.5">
    {#each system as item}
      <SystemCard icon={item.icon} label={item.label} value={item.value} />
    {/each}
  </div>

  <div class="grid grid-cols-[repeat(auto-fit,minmax(170px,1fr))] gap-3.5 mb-4">
    <AppCard icon={Download} title={$_("home.cards.install.title")} description={$_("home.cards.install.description")}>
      <svelte:fragment slot="footer">
        <span class="text-[0.72rem] px-2.5 py-1 rounded-full font-semibold whitespace-nowrap bg-success/15 text-success">
          {$_("home.cards.install.badge")}
        </span>
        <button
          class="bg-primary hover:bg-primary-hover text-text-bright px-3 py-1.5 rounded font-semibold text-[0.82rem]"
          on:click={() => navigate("install")}
        >
          {$_("home.cards.install.action")}
        </button>
      </svelte:fragment>
    </AppCard>
    <AppCard icon={Wrench} title={$_("home.cards.rescue.title")} description={$_("home.cards.rescue.description")}>
      <svelte:fragment slot="footer">
        <button
          class="ml-auto bg-primary hover:bg-primary-hover text-text-bright px-3 py-1.5 rounded font-semibold text-[0.82rem]"
          on:click={() => navigate("rescue")}
        >
          {$_("home.cards.rescue.action")}
        </button>
      </svelte:fragment>
    </AppCard>
    <AppCard icon={LayoutGrid} title={$_("home.cards.tools.title")} description={$_("home.cards.tools.description")}>
      <svelte:fragment slot="footer">
        <button
          class="ml-auto bg-primary hover:bg-primary-hover text-text-bright px-3 py-1.5 rounded font-semibold text-[0.82rem]"
          on:click={() => navigate("tools")}
        >
          {$_("home.cards.tools.action")}
        </button>
      </svelte:fragment>
    </AppCard>
  </div>

  <div class="flex flex-wrap items-center justify-center gap-2.5 mt-auto">
    {#each links as link}
      <button
        class="flex items-center gap-2 px-4 py-2 rounded border border-border text-text hover:bg-surface hover:text-text-bright text-[0.88rem] transition-colors"
        on:click={() => open(link.url)}
      >
        <svelte:component this={link.icon} size={16} />
        <span>{link.label}</span>
      </button>
    {/each}
  </div>
</div>
