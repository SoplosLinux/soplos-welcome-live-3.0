<script lang="ts">
  import { onDestroy } from "svelte";
  import { _ } from "svelte-i18n";
  import { api } from "../lib/api";
  import { progress } from "../lib/progress";
  import { Power, KeyRound, RefreshCw, ChevronRight, ChevronLeft, HardDrive, Check, TerminalSquare } from "lucide-svelte";

  type Problem = "boot" | "password" | "update" | "terminal";
  type Phase = "choose" | "scanning" | "none" | "pick" | "working" | "done" | "failed" | "terminal";

  interface FoundSystem {
    device: string;
    name: string;
    size: string;
    fstype: string;
    subvol: string | null;
    disk: string;
    users: string[];
  }

  let phase: Phase = "choose";
  let problem: Problem = "boot";
  let systems: FoundSystem[] = [];
  let selected = 0;
  let user = "";
  let password = "";
  let repeat = "";
  let formError = "";
  let steps: { id: string; label: string; state: "todo" | "doing" | "done" }[] = [];
  let log = "";
  let failure = "";
  let showDetails = false;

  $: system = systems[selected] as FoundSystem | undefined;
  $: if (system && !system.users.includes(user)) user = system.users[0] ?? "";

  // Leaving the view with something mounted would leave the disk busy.
  onDestroy(() => {
    if (phase === "working") return;
    progress.set(null);
  });

  function reset() {
    phase = "choose";
    systems = [];
    selected = 0;
    password = "";
    repeat = "";
    formError = "";
    log = "";
    failure = "";
    showDetails = false;
    progress.set(null);
  }

  async function begin(p: Problem) {
    problem = p;
    phase = "scanning";
    progress.set("busy");
    try {
      systems = await api.invoke<FoundSystem[]>("rescue_scan");
    } catch (error) {
      systems = [];
      log += `${error}\n`;
    }
    progress.set(null);
    selected = 0;
    phase = systems.length ? "pick" : "none";
  }

  function plan(p: Problem): string[] {
    switch (p) {
      case "boot":
        return ["mount", "grub_install", "update_grub", "unmount"];
      case "update":
        return ["mount", "initramfs", "update_grub", "unmount"];
      case "password":
        return ["mount", "password", "unmount"];
      default:
        return ["mount"];
    }
  }

  function addLog(text: string) {
    if (text.trim()) log += `${text.trim()}\n`;
  }

  async function run() {
    if (!system) return;
    if (problem === "password") {
      if (!user || !password) {
        formError = $_("rescue.form.empty");
        return;
      }
      if (password !== repeat) {
        formError = $_("rescue.form.mismatch");
        return;
      }
    }
    formError = "";
    const ids = plan(problem);
    steps = ids.map((id) => ({ id, label: $_(`rescue.steps.${id}`), state: "todo" }));
    phase = "working";
    log = "";
    failure = "";
    progress.set(0);

    let mounted = false;
    try {
      for (let i = 0; i < steps.length; i++) {
        steps[i].state = "doing";
        steps = steps;
        progress.set(i / steps.length);
        const id = steps[i].id;
        if (id === "mount") {
          addLog(await api.invoke<string>("rescue_mount", { device: system.device, subvol: system.subvol }));
          mounted = true;
        } else if (id === "unmount") {
          await api.invoke("rescue_unmount");
          mounted = false;
        } else {
          addLog(
            await api.invoke<string>("rescue_step", {
              step: id,
              device: system.device,
              user: id === "password" ? user : null,
              password: id === "password" ? password : null,
            }),
          );
        }
        steps[i].state = "done";
        steps = steps;
      }
      if (problem === "terminal") {
        await api.invoke("rescue_open_terminal", {
          title: $_("rescue.terminal.title"),
          exitMessage: $_("rescue.terminal.exit"),
          cleanupMessage: $_("rescue.terminal.cleanup"),
          doneMessage: $_("rescue.terminal.done"),
        });
        phase = "terminal";
      } else {
        phase = "done";
      }
    } catch (error) {
      failure = String(error);
      addLog(failure);
      if (mounted && problem !== "terminal") {
        try {
          await api.invoke("rescue_unmount");
        } catch {
          // Nothing else to do: the failure is already reported.
        }
      }
      phase = "failed";
    } finally {
      password = "";
      repeat = "";
      progress.set(null);
    }
  }

  async function reboot() {
    try {
      await api.invoke("rescue_reboot");
    } catch (error) {
      console.error("Failed to reboot", error);
    }
  }

  const problems: { id: Problem; icon: typeof Power }[] = [
    { id: "boot", icon: Power },
    { id: "password", icon: KeyRound },
    { id: "update", icon: RefreshCw },
  ];
</script>

<div class="max-w-[640px]">
  {#if phase === "choose"}
    <p class="text-[0.95rem] text-text-bright mb-3.5">{$_("rescue.question")}</p>
    <div class="flex flex-col gap-2.5">
      {#each problems as item}
        <button
          class="flex items-center gap-3.5 px-4 py-3.5 rounded border border-border bg-surface text-left hover:border-primary transition-colors"
          on:click={() => begin(item.id)}
        >
          <div class="w-[42px] h-[42px] bg-surface-2 rounded-lg grid place-items-center text-primary shrink-0">
            <svelte:component this={item.icon} size={22} />
          </div>
          <div class="min-w-0">
            <div class="text-[0.96rem] font-semibold text-text-bright">{$_(`rescue.problems.${item.id}.title`)}</div>
            <div class="text-[0.8rem] text-text-muted">{$_(`rescue.problems.${item.id}.description`)}</div>
          </div>
          <ChevronRight size={18} class="ml-auto text-text-muted shrink-0" />
        </button>
      {/each}
    </div>
    <button
      class="mt-5 flex items-center gap-2 text-[0.8rem] text-text-muted hover:text-text-bright hover:underline"
      on:click={() => begin("terminal")}
    >
      <TerminalSquare size={14} />
      {$_("rescue.advanced")}
    </button>
  {:else if phase === "scanning"}
    <p class="text-[0.95rem] text-text-bright">{$_("rescue.scanning")}</p>
    <p class="text-[0.8rem] text-text-muted mt-1">{$_("rescue.scanningHint")}</p>
  {:else if phase === "none"}
    <p class="text-[1rem] font-semibold text-text-bright mb-1">{$_("rescue.none.title")}</p>
    <p class="text-[0.85rem] text-text-muted mb-4">{$_("rescue.none.message")}</p>
    <div class="flex gap-2.5">
      <button
        class="bg-primary hover:bg-primary-hover text-text-bright px-5 py-2 rounded font-semibold text-[0.85rem]"
        on:click={() => begin(problem)}
      >
        {$_("common.retry")}
      </button>
      <button class="px-4 py-2 rounded border border-border text-text hover:bg-surface text-[0.85rem]" on:click={reset}>
        {$_("common.back")}
      </button>
    </div>
  {:else if phase === "pick" && system}
    <button class="flex items-center gap-1 text-[0.8rem] text-text-muted hover:text-text-bright mb-3.5" on:click={reset}>
      <ChevronLeft size={14} />
      {$_("common.back")}
    </button>
    <p class="text-[0.95rem] text-text-bright mb-3">
      {systems.length === 1 ? $_("rescue.foundOne") : $_("rescue.chooseSystem")}
    </p>
    <div class="flex flex-col gap-2 mb-4">
      {#each systems as item, index}
        <button
          class="flex items-center gap-3.5 px-4 py-3 rounded border text-left transition-colors
            {selected === index ? 'border-primary bg-primary/10' : 'border-border bg-surface hover:border-primary'}"
          on:click={() => (selected = index)}
        >
          <HardDrive size={22} class="text-primary shrink-0" />
          <div class="min-w-0">
            <div class="text-[0.92rem] font-semibold text-text-bright truncate">{item.name}</div>
            <div class="text-[0.76rem] text-text-muted truncate">{item.disk} · {item.size} · {item.fstype}</div>
          </div>
          {#if selected === index}<Check size={18} class="ml-auto text-primary shrink-0" />{/if}
        </button>
      {/each}
    </div>

    {#if problem === "password"}
      <div class="flex flex-col gap-2.5 max-w-[340px] mb-4">
        <label class="text-[0.76rem] text-text-muted" for="rescue-user">{$_("rescue.form.user")}</label>
        <select
          id="rescue-user"
          class="bg-surface-2 border border-border rounded px-2.5 py-2 text-[0.86rem] text-text-bright outline-none focus-visible:ring-2 focus-visible:ring-primary/60"
          bind:value={user}
        >
          {#each system.users as name}
            <option value={name}>{name}</option>
          {/each}
        </select>
        <label class="text-[0.76rem] text-text-muted" for="rescue-pass">{$_("rescue.form.password")}</label>
        <input
          id="rescue-pass"
          type="password"
          autocomplete="off"
          class="bg-surface-2 border border-border rounded px-2.5 py-2 text-[0.86rem] text-text-bright outline-none focus-visible:ring-2 focus-visible:ring-primary/60"
          bind:value={password}
        />
        <label class="text-[0.76rem] text-text-muted" for="rescue-repeat">{$_("rescue.form.repeat")}</label>
        <input
          id="rescue-repeat"
          type="password"
          autocomplete="off"
          class="bg-surface-2 border border-border rounded px-2.5 py-2 text-[0.86rem] text-text-bright outline-none focus-visible:ring-2 focus-visible:ring-primary/60"
          bind:value={repeat}
        />
        {#if formError}<p class="text-[0.78rem] text-danger">{formError}</p>{/if}
      </div>
    {/if}

    <button
      class="bg-primary hover:bg-primary-hover text-text-bright px-6 py-2.5 rounded font-semibold text-[0.88rem]"
      on:click={run}
    >
      {$_(`rescue.action.${problem}`)}
    </button>
  {:else if phase === "working"}
    <p class="text-[0.95rem] text-text-bright mb-4">{$_(`rescue.working.${problem}`)}</p>
    <ul class="flex flex-col gap-2.5 mb-4">
      {#each steps as step}
        <li
          class="flex items-center gap-2.5 text-[0.9rem] {step.state === 'doing'
            ? 'text-primary font-semibold'
            : step.state === 'done'
              ? 'text-text-bright'
              : 'text-text-muted'}"
        >
          <span
            class="w-5 h-5 rounded-full grid place-items-center shrink-0 border-2 {step.state === 'done'
              ? 'bg-success border-success text-white'
              : step.state === 'doing'
                ? 'border-primary'
                : 'border-border'}"
          >
            {#if step.state === "done"}<Check size={12} />{/if}
          </span>
          {step.label}
        </li>
      {/each}
    </ul>
  {:else if phase === "done"}
    <div class="text-center py-6">
      <div class="w-16 h-16 mx-auto mb-3 rounded-full bg-success/15 text-success grid place-items-center">
        <Check size={32} />
      </div>
      <h2 class="text-[1.2rem] font-semibold text-text-bright mb-1">{$_(`rescue.done.${problem}.title`)}</h2>
      <p class="text-[0.86rem] text-text-muted mb-4">{$_(`rescue.done.${problem}.message`)}</p>
      <div class="flex justify-center gap-2.5">
        <button
          class="bg-primary hover:bg-primary-hover text-text-bright px-5 py-2 rounded font-semibold text-[0.85rem]"
          on:click={reboot}
        >
          {$_("rescue.reboot")}
        </button>
        <button class="px-4 py-2 rounded border border-border text-text hover:bg-surface text-[0.85rem]" on:click={reset}>
          {$_("common.back")}
        </button>
      </div>
    </div>
  {:else if phase === "terminal"}
    <div class="text-center py-6">
      <h2 class="text-[1.1rem] font-semibold text-text-bright mb-1">{$_("rescue.terminalOpen.title")}</h2>
      <p class="text-[0.86rem] text-text-muted mb-4">{$_("rescue.terminalOpen.message")}</p>
      <button class="px-4 py-2 rounded border border-border text-text hover:bg-surface text-[0.85rem]" on:click={reset}>
        {$_("common.back")}
      </button>
    </div>
  {:else if phase === "failed"}
    <div class="py-4">
      <h2 class="text-[1.1rem] font-semibold text-danger mb-1">{$_("rescue.failed.title")}</h2>
      <p class="text-[0.86rem] text-text-muted mb-3">{$_("rescue.failed.message")}</p>
      <div class="flex gap-2.5 mb-3">
        <button
          class="bg-primary hover:bg-primary-hover text-text-bright px-5 py-2 rounded font-semibold text-[0.85rem]"
          on:click={() => begin(problem)}
        >
          {$_("common.retry")}
        </button>
        <button class="px-4 py-2 rounded border border-border text-text hover:bg-surface text-[0.85rem]" on:click={reset}>
          {$_("common.back")}
        </button>
      </div>
      <button class="text-[0.78rem] text-text-muted hover:text-text-bright hover:underline" on:click={() => (showDetails = !showDetails)}>
        {showDetails ? $_("common.hideDetails") : $_("common.showDetails")}
      </button>
      {#if showDetails}
        <pre class="mt-2 p-3 rounded border border-border bg-panel text-[0.72rem] text-text overflow-x-auto whitespace-pre-wrap">{log}</pre>
      {/if}
    </div>
  {/if}
</div>
