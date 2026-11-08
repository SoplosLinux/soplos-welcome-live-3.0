<script lang="ts">
  import { _ } from "svelte-i18n";
  import { Power, KeyRound, RefreshCw } from "lucide-svelte";

  // The help column of the Rescue page: one card for each option, in the
  // order of the options, with what it does and what it touches.
  const options = [
    { id: "boot", icon: Power },
    { id: "password", icon: KeyRound },
    { id: "update", icon: RefreshCw },
  ];
</script>

<aside class="flex flex-col gap-2.5 text-[0.78rem] text-text">
  {#each options as option}
    <div class="rounded-xl border border-border bg-surface px-3.5 py-3">
      <h3 class="flex items-center gap-1.5 mb-2 text-[0.66rem] font-semibold tracking-[0.07em] text-text-muted uppercase">
        <svelte:component this={option.icon} size={12} class="text-primary" />
        {$_(`rescue.problems.${option.id}.title`)}
      </h3>
      <p class="leading-relaxed">{$_(`rescue.hints.${option.id}`)}</p>
    </div>
  {/each}
</aside>
