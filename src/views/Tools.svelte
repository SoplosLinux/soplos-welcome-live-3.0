<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from "svelte-i18n";
  import AppCard from "../components/AppCard.svelte";
  import Modal from "../components/Modal.svelte";
  import { api } from "../lib/api";
  import { HardDrive, Cpu, Monitor, Hash } from "lucide-svelte";

  interface HardwareInfo {
    cpu: string;
    memory: string;
    disks: { name: string; size: string; model: string }[];
  }
  interface ResolutionInfo {
    available: string[];
    current: string | null;
    supported: boolean;
  }

  let edition = "unknown";
  let hardware: HardwareInfo | null = null;
  let hardwareOpen = false;
  let resolution: ResolutionInfo | null = null;
  let resolutionError = false;
  let numlock = true;
  let numlockError = "";
  let message: { title: string; text: string } | null = null;

  onMount(async () => {
    try {
      const env = await api.invoke<{ edition: string }>("get_environment_info");
      edition = env.edition;
    } catch {
      edition = "unknown";
    }
    try {
      resolution = await api.invoke<ResolutionInfo>("get_resolution_info");
    } catch {
      resolution = null;
    }
    if (edition === "tyron") {
      try {
        numlock = await api.invoke<boolean>("get_numlock_enabled");
      } catch {
        numlock = true;
      }
    }
  });

  async function openGparted() {
    try {
      await api.invoke("open_gparted");
    } catch (error) {
      message = { title: $_("common.error"), text: $_("tools.gparted.error", { values: { error: String(error) } }) };
    }
  }

  async function showHardware() {
    try {
      hardware = await api.invoke<HardwareInfo>("get_hardware_info");
      hardwareOpen = true;
    } catch (error) {
      message = { title: $_("common.error"), text: String(error) };
    }
  }

  async function changeResolution(e: Event) {
    const value = (e.currentTarget as HTMLSelectElement).value;
    resolutionError = false;
    try {
      await api.invoke("set_resolution", { resolution: value });
      if (resolution) resolution = { ...resolution, current: value };
    } catch {
      resolutionError = true;
    }
  }

  async function toggleNumlock() {
    const next = !numlock;
    numlockError = "";
    try {
      await api.invoke("set_numlock_enabled", { enabled: next });
      numlock = next;
      message = next
        ? { title: $_("tools.numlock.enabledTitle"), text: $_("tools.numlock.enabledText") }
        : { title: $_("tools.numlock.disabledTitle"), text: $_("tools.numlock.disabledText") };
    } catch (error) {
      numlockError = String(error);
      message = { title: $_("common.error"), text: numlockError };
    }
  }
</script>

<div class="grid grid-cols-1 md:grid-cols-2 gap-3.5">
  <AppCard icon={HardDrive} title={$_("tools.gparted.title")} description={$_("tools.gparted.description")}>
    <svelte:fragment slot="footer">
      <span></span>
      <button
        class="bg-primary hover:bg-primary-hover text-text-bright px-3 py-1.5 rounded font-semibold text-[0.82rem]"
        on:click={openGparted}
      >
        {$_("common.open")}
      </button>
    </svelte:fragment>
  </AppCard>

  <AppCard icon={Cpu} title={$_("tools.hardware.title")} description={$_("tools.hardware.description")}>
    <svelte:fragment slot="footer">
      <span></span>
      <button
        class="bg-primary hover:bg-primary-hover text-text-bright px-3 py-1.5 rounded font-semibold text-[0.82rem]"
        on:click={showHardware}
      >
        {$_("tools.hardware.view")}
      </button>
    </svelte:fragment>
  </AppCard>

  <AppCard icon={Monitor} title={$_("tools.resolution.title")} description={$_("tools.resolution.description")}>
    <svelte:fragment slot="footer">
      {#if resolution && resolution.supported && resolution.available.length}
        <select
          class="appearance-none bg-surface-2 border border-border rounded px-2.5 py-1.5 text-[0.82rem] text-text-bright outline-none focus-visible:ring-2 focus-visible:ring-primary/60"
          value={resolution.current ?? ""}
          on:change={changeResolution}
          aria-label={$_("tools.resolution.title")}
        >
          {#each resolution.available as item}
            <option value={item}>{item.replace("x", " × ")}</option>
          {/each}
        </select>
        {#if resolutionError}<span class="text-[0.74rem] text-danger">{$_("tools.resolution.error")}</span>{/if}
      {:else}
        <span class="text-[0.76rem] text-text-muted">{$_("tools.resolution.unsupported")}</span>
      {/if}
    </svelte:fragment>
  </AppCard>

  {#if edition === "tyron"}
    <AppCard icon={Hash} title={$_("tools.numlock.title")} description={$_("tools.numlock.description")}>
      <svelte:fragment slot="footer">
        <span class="text-[0.72rem] px-2.5 py-1 rounded-full font-semibold bg-surface-2 text-text-muted">
          {$_("tools.numlock.badge")}
        </span>
        <button
          class="relative w-[36px] h-[20px] rounded-full transition-colors shrink-0 {numlock ? 'bg-primary' : 'bg-surface-2'}"
          on:click={toggleNumlock}
          aria-pressed={numlock}
          aria-label={$_("tools.numlock.title")}
        >
          <span
            class="absolute top-[2px] left-[2px] w-[16px] h-[16px] rounded-full bg-text-bright transition-transform {numlock
              ? 'translate-x-[16px]'
              : ''}"
          ></span>
        </button>
      </svelte:fragment>
    </AppCard>
  {/if}
</div>

{#if hardwareOpen && hardware}
  <Modal title={$_("tools.hardware.title")} onClose={() => (hardwareOpen = false)} confirmLabel={$_("common.close")}>
    <dl class="flex flex-col gap-2.5">
      <div>
        <dt class="text-[0.72rem] uppercase tracking-wider text-text-muted">{$_("tools.hardware.cpu")}</dt>
        <dd class="text-text-bright">{hardware.cpu || $_("common.unknown")}</dd>
      </div>
      <div>
        <dt class="text-[0.72rem] uppercase tracking-wider text-text-muted">{$_("tools.hardware.memory")}</dt>
        <dd class="text-text-bright">{hardware.memory || $_("common.unknown")}</dd>
      </div>
      <div>
        <dt class="text-[0.72rem] uppercase tracking-wider text-text-muted">{$_("tools.hardware.storage")}</dt>
        {#each hardware.disks as disk}
          <dd class="text-text-bright">{disk.name} · {disk.size}{disk.model ? ` · ${disk.model}` : ""}</dd>
        {:else}
          <dd class="text-text-bright">{$_("common.unknown")}</dd>
        {/each}
      </div>
    </dl>
  </Modal>
{/if}

{#if message}
  <Modal title={message.title} onClose={() => (message = null)} confirmLabel={$_("common.ok")}>
    <p class="whitespace-pre-line">{message.text}</p>
  </Modal>
{/if}
