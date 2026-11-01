<script lang="ts">
  import type { ComponentType } from "svelte";

  export let icon: ComponentType;
  export let label: string;
  export let value: string;
</script>

<div class="bg-surface border border-border rounded p-3 min-w-0 flex items-center gap-3">
  <div class="w-8 h-8 rounded-lg bg-surface-2 grid place-items-center text-primary shrink-0">
    <svelte:component this={icon} size={18} />
  </div>
  <div class="min-w-0">
    <div class="text-[0.75rem] text-text-muted mb-0.5">{label}</div>
    <div class="text-[0.85rem] font-medium text-text-bright leading-snug">{value}</div>
  </div>
</div>
