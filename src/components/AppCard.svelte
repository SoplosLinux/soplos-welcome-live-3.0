<script lang="ts">
  import type { ComponentType } from "svelte";

  export let icon: ComponentType;
  export let title: string;
  export let description: string;
</script>

<div class="h-full flex flex-col bg-surface border border-border rounded p-[18px] transition-colors hover:border-primary">
  <div class="flex items-center gap-3 mb-2.5">
    <div class="w-[38px] h-[38px] bg-surface-2 rounded-lg grid place-items-center text-primary shrink-0">
      <svelte:component this={icon} size={19} />
    </div>
    <h3 class="text-[0.95rem] font-semibold text-text-bright">{title}</h3>
  </div>
  <p class="text-[0.82rem] text-text-muted leading-snug mb-3.5">{description}</p>
  <div class="mt-auto flex flex-wrap items-center justify-between gap-2">
    <slot name="footer" />
  </div>
</div>
