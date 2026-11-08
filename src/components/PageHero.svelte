<script lang="ts">
  import type { ComponentType } from "svelte";

  // The banner over every page but Home: the image of the welcome screen, the
  // icon of the page and its title.
  export let title: string;
  export let icon: ComponentType;
</script>

<div
  class="rounded px-5 py-8 mb-4 flex items-center gap-3.5 border border-border"
  style="background: linear-gradient(rgba(20, 20, 22, 0.55), rgba(20, 20, 22, 0.82)),
    url('/assets/hero/welcome-live-background.jpg'); background-size: cover; background-position: center;"
>
  <div class="w-[42px] h-[42px] rounded-lg grid place-items-center bg-white/15 text-white shrink-0">
    <svelte:component this={icon} size={22} />
  </div>
  <h2 class="text-[1.3rem] font-bold text-white">{title}</h2>
</div>
