<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { _ } from "svelte-i18n";
  import Modal from "../components/Modal.svelte";
  import { api } from "../lib/api";
  import { progress } from "../lib/progress";
  import { Download } from "lucide-svelte";

  interface InstallerInfo {
    name: string;
    available: boolean;
  }
  interface InstallChecks {
    uefi: boolean;
    disks: { name: string; size: string }[];
    onBattery: boolean | null;
  }

  let installer: InstallerInfo | null = null;
  let checks: InstallChecks | null = null;
  interface Readiness {
    internet: boolean;
    clockOffset: number | null;
  }

  // The installer stops with error 100 without a connection to the servers or
  // with a clock that does not match theirs (a few minutes are tolerated).
  const CLOCK_TOLERANCE = 300;

  let internet: boolean | null = null;
  let clockOffset: number | null = null;
  let fixing = false;
  let fixError = "";
  let warnOpen = false;
  let launching = false;
  let errorOpen = false;
  let timer: ReturnType<typeof setInterval> | undefined;

  async function checkReadiness() {
    try {
      const r = await api.invoke<Readiness>("check_readiness");
      internet = r.internet;
      clockOffset = r.clockOffset;
    } catch {
      internet = false;
      clockOffset = null;
    }
  }

  $: clockBad = clockOffset !== null && Math.abs(clockOffset) > CLOCK_TOLERANCE;
  $: clockMinutes = clockOffset === null ? 0 : Math.round(Math.abs(clockOffset) / 60);

  async function fixClock() {
    fixing = true;
    fixError = "";
    try {
      await api.invoke("fix_clock");
    } catch (error) {
      fixError = String(error);
    }
    await checkReadiness();
    fixing = false;
  }

  onMount(async () => {
    try {
      installer = await api.invoke<InstallerInfo>("get_installer_info");
    } catch {
      installer = null;
    }
    try {
      checks = await api.invoke<InstallChecks>("get_install_checks");
    } catch {
      checks = null;
    }
    await checkReadiness();
    // The same rhythm as the status indicator of 2.x: every 15 seconds.
    timer = setInterval(checkReadiness, 15000);
  });

  onDestroy(() => timer && clearInterval(timer));

  // Asks before starting when the installer is going to fail.
  function tryStart() {
    if (internet === false || clockBad) warnOpen = true;
    else start();
  }

  async function start() {
    warnOpen = false;
    launching = true;
    progress.set("busy");
    try {
      await api.invoke("launch_installer");
      // The installer takes over: this window closes a second later, as in 2.x.
      setTimeout(() => getCurrentWindow().close(), 1000);
    } catch {
      launching = false;
      progress.set(null);
      errorOpen = true;
    }
  }

  type Row = { label: string; value: string; state: "ok" | "warn" };

  $: rows = [
    {
      label: $_("install.checks.internet"),
      value: internet === null ? "…" : internet ? $_("install.checks.internetOn") : $_("install.checks.internetOff"),
      state: internet === false ? "warn" : "ok",
    },
    ...(clockOffset === null
      ? []
      : [
          {
            label: $_("install.checks.clock"),
            value: clockBad ? $_("install.checks.clockOff", { values: { minutes: clockMinutes } }) : $_("install.checks.clockOk"),
            state: clockBad ? "warn" : "ok",
          },
        ]),
    ...(checks
      ? ([
          {
            label: $_("install.checks.uefi"),
            value: checks.uefi ? $_("install.checks.uefiOn") : $_("install.checks.uefiOff"),
            state: "ok",
          },
          {
            label: $_("install.checks.disk"),
            value: checks.disks.length
              ? checks.disks.map((d) => `${d.name} · ${d.size}`).join(", ")
              : $_("install.checks.diskNone"),
            state: checks.disks.length ? "ok" : "warn",
          },
          ...(checks.onBattery === null
            ? []
            : [
                {
                  label: $_("install.checks.power"),
                  value: checks.onBattery ? $_("install.checks.powerBattery") : $_("install.checks.powerOk"),
                  state: checks.onBattery ? "warn" : "ok",
                },
              ]),
        ] as Row[])
      : []),
  ] as Row[];
</script>

<div class="flex flex-col gap-4">
  <div class="bg-surface border border-border rounded p-[18px]">
    <div class="flex items-center gap-3 mb-2.5">
      <div class="w-[38px] h-[38px] bg-surface-2 rounded-lg grid place-items-center text-primary shrink-0">
        <Download size={19} />
      </div>
      <h3 class="text-[0.95rem] font-semibold text-text-bright">
        {$_("install.intro.title", { values: { name: installer?.name ?? "" } })}
      </h3>
    </div>
    <p class="text-[0.82rem] text-text-muted leading-snug">{$_("install.intro.description")}</p>
  </div>

  <div>
    <h3 class="text-[0.72rem] font-semibold uppercase tracking-wider text-text-muted mb-2.5">
      {$_("install.checks.title")}
    </h3>
    <ul class="flex flex-col gap-1.5">
      {#each rows as row}
        <li class="flex items-center gap-2.5 px-3 py-2 rounded bg-surface border border-border text-[0.84rem]">
          <span class="w-2 h-2 rounded-full shrink-0 {row.state === 'ok' ? 'bg-success' : 'bg-warning'}"></span>
          <span class="text-text-bright">{row.label}</span>
          <span class="ml-auto text-text-muted text-[0.78rem] text-right">{row.value}</span>
        </li>
      {/each}
    </ul>
  </div>

  {#if clockBad}
    <div class="bg-surface border border-border rounded p-[14px] flex items-center gap-3">
      <p class="text-[0.82rem] text-text-muted leading-snug flex-1">{$_("install.clock.hint")}</p>
      <button
        class="border border-border hover:border-primary text-text-bright px-4 py-2 rounded text-[0.84rem] disabled:opacity-50 shrink-0"
        disabled={fixing}
        on:click={fixClock}
      >
        {fixing ? $_("install.clock.fixing") : $_("install.clock.fix")}
      </button>
    </div>
  {/if}

  <div>
    <button
      class="bg-primary hover:bg-primary-hover text-text-bright px-6 py-2.5 rounded font-semibold text-[0.9rem] disabled:opacity-50"
      disabled={launching || (installer !== null && !installer.available)}
      on:click={tryStart}
    >
      {launching ? $_("install.starting") : $_("install.start")}
    </button>
  </div>
</div>

{#if warnOpen}
  <Modal
    variant="warning"
    title={$_("install.warn.title")}
    maxWidth={500}
    confirmLabel={$_("install.warn.anyway")}
    onConfirm={start}
    onClose={() => (warnOpen = false)}
  >
    {#if internet === false}
      <p class="mb-2">{$_("install.warn.internet")}</p>
    {/if}
    {#if clockBad}
      <p class="mb-2">{$_("install.warn.clock", { values: { minutes: clockMinutes } })}</p>
    {/if}
    <p class="text-text-muted">{$_("install.warn.error100")}</p>
  </Modal>
{/if}

{#if fixError}
  <Modal variant="warning" title={$_("common.error")} onClose={() => (fixError = "")}>
    <p class="whitespace-pre-line">{fixError}</p>
  </Modal>
{/if}

{#if errorOpen}
  <Modal variant="warning" title={$_("install.notFound.title")} onClose={() => (errorOpen = false)}>
    {$_("install.notFound.message", { values: { name: installer?.name ?? "" } })}
  </Modal>
{/if}
