<script lang="ts">
  import { fade, scale } from "svelte/transition";
  import { _ } from "svelte-i18n";
  import { Info, HelpCircle, AlertTriangle, ShieldAlert } from "lucide-svelte";
  import { onMount } from "svelte";
  import { autofocus } from "../lib/autofocus";
  import { modalDepth } from "../lib/ui";

  // info: single OK button. confirm/warning/danger: Cancel + confirm button
  // (danger and warning also tint the header icon and the confirm button).
  export let variant: "info" | "confirm" | "warning" | "danger" = "info";
  export let title = "";
  export let onClose: () => void;
  export let onConfirm: (() => void) | undefined = undefined;
  export let confirmLabel = "";
  export let cancelLabel = "";
  export let maxWidth = 440;
  // Choice popups put their own buttons in the body and skip the footer.
  export let showFooter = true;
  export let confirmDisabled = false;

  // Counted while open, so the window's shortcuts wait until it closes.
  onMount(() => {
    modalDepth.update((n) => n + 1);
    return () => modalDepth.update((n) => n - 1);
  });

  const icons = { info: Info, confirm: HelpCircle, warning: AlertTriangle, danger: ShieldAlert };
  const iconTint = {
    info: "bg-primary/15 text-primary",
    confirm: "bg-primary/15 text-primary",
    warning: "bg-warning/15 text-warning",
    danger: "bg-danger/15 text-danger",
  };

  $: hasConfirm = variant !== "info";
  $: confirmClass =
    variant === "danger"
      ? "bg-danger hover:bg-danger/85 text-text-bright"
      : "bg-primary hover:bg-primary-hover text-text-bright";

  function accept() {
    if (hasConfirm && onConfirm) onConfirm();
    else onClose();
  }

  // Short opacity/scale only, no backdrop-filter: blur is expensive on the
  // software-rendered WebKitGTK used in VMs.
  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
    else if (e.key === "Enter" && e.target === e.currentTarget) accept();
  }
</script>

<div
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
  role="button"
  tabindex="-1"
  transition:fade|global={{ duration: 120 }}
  on:click={onClose}
  on:keydown={onKeydown}
>
  <div
    class="w-full bg-surface border border-border rounded-xl shadow-2xl overflow-hidden"
    style="max-width: {maxWidth}px;"
    role="presentation"
    transition:scale|global={{ duration: 140, start: 0.96 }}
    on:click|stopPropagation
  >
    <div class="flex items-start gap-3 px-5 pt-5 pb-3">
      <div class="w-9 h-9 rounded-full grid place-items-center shrink-0 {iconTint[variant]}">
        <svelte:component this={icons[variant]} size={19} />
      </div>
      <div class="min-w-0 flex-1 pt-1">
        {#if title}
          <h3 class="text-text-bright font-semibold text-[1rem] leading-snug">{title}</h3>
        {/if}
      </div>
    </div>

    <div class="px-5 pb-4 text-[0.85rem] text-text leading-relaxed">
      <slot />
    </div>

    {#if showFooter}
      <div class="flex justify-end gap-2 px-5 py-3.5 bg-surface-2/50 border-t border-border">
        {#if hasConfirm}
          <button
            class="px-4 py-2 rounded border border-border text-text hover:bg-surface-2 text-[0.85rem]"
            on:click={onClose}
          >
            {cancelLabel || $_("common.cancel")}
          </button>
        {/if}
        <button
          class="px-5 py-2 rounded text-[0.85rem] font-semibold {hasConfirm ? confirmClass : 'bg-primary hover:bg-primary-hover text-text-bright'}"
          use:autofocus
          disabled={confirmDisabled}
          on:click={accept}
        >
          {confirmLabel || $_("common.ok")}
        </button>
      </div>
    {:else}
      <div class="h-1"></div>
    {/if}
  </div>
</div>
