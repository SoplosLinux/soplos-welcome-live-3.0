<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from "svelte-i18n";
  import Modal from "../components/Modal.svelte";
  import { api } from "../lib/api";
  import { progress } from "../lib/progress";
  import { LOCALE_NAMES, type SupportedLocale } from "../i18n";

  // One keyboard layout per language of the dictionaries, the same pairs 2.x
  // used: English is the US layout.
  const LAYOUT_OF: Record<SupportedLocale, string> = {
    es: "es",
    en: "us",
    de: "de",
    fr: "fr",
    it: "it",
    pt: "pt",
    ro: "ro",
    ru: "ru",
  };
  const ORDER: SupportedLocale[] = ["es", "en", "de", "fr", "it", "pt", "ro", "ru"];

  let language: SupportedLocale = "en";
  let layout = "us";
  let current: SupportedLocale = "en";
  let currentLayout = "us";
  // Until the keyboard is chosen by hand it follows the language.
  let layoutTouched = false;
  let applying = false;
  let stepLabel = "";
  let errorText = "";

  onMount(async () => {
    try {
      const state = await api.invoke<{ language: SupportedLocale; layout: string }>("get_language_state");
      language = current = state.language;
      layout = currentLayout = state.layout;
    } catch {
      // The defaults stay.
    }
  });

  function pick(code: SupportedLocale) {
    language = code;
    if (!layoutTouched) layout = LAYOUT_OF[code];
  }

  $: unchanged = language === current && layout === currentLayout;

  async function apply() {
    applying = true;
    errorText = "";
    const steps = ["locale", "user", "xdg", "bookmarks", "restart"];
    try {
      for (let i = 0; i < steps.length; i++) {
        stepLabel = $_(`language.steps.${steps[i]}`);
        progress.set(i / steps.length);
        await api.invoke("language_step", { step: steps[i], lang: language, layout });
      }
      progress.set(1);
    } catch (error) {
      errorText = String(error);
      progress.set(null);
      applying = false;
    }
  }
</script>

<div class="max-w-[640px]">
  <p class="text-[0.85rem] text-text-muted mb-4">{$_("language.description")}</p>

  <h3 class="text-[0.72rem] font-semibold uppercase tracking-wider text-text-muted mb-2.5">{$_("language.languages")}</h3>
  <div class="grid grid-cols-2 sm:grid-cols-3 gap-2 mb-5">
    {#each ORDER as code}
      <button
        class="px-3 py-2.5 rounded border text-left text-[0.88rem] transition-colors disabled:opacity-60
          {language === code
          ? 'border-primary bg-primary/10 text-primary font-semibold'
          : 'border-border bg-surface text-text hover:border-primary'}"
        aria-pressed={language === code}
        disabled={applying}
        on:click={() => pick(code)}
      >
        {LOCALE_NAMES[code]}
      </button>
    {/each}
  </div>

  <h3 class="text-[0.72rem] font-semibold uppercase tracking-wider text-text-muted mb-2.5">{$_("language.keyboard")}</h3>
  <select
    class="appearance-none bg-surface-2 border border-border rounded px-3 py-2 text-[0.86rem] text-text-bright outline-none focus-visible:ring-2 focus-visible:ring-primary/60 min-w-[220px] disabled:opacity-60"
    bind:value={layout}
    disabled={applying}
    on:change={() => (layoutTouched = true)}
    aria-label={$_("language.keyboard")}
  >
    {#each ORDER as code}
      <option value={LAYOUT_OF[code]}>{LOCALE_NAMES[code]}{code === "en" ? " (US)" : ""}</option>
    {/each}
  </select>
  <p class="text-[0.76rem] text-text-muted mt-1.5 mb-5">{$_("language.keyboardHint")}</p>

  <button
    class="bg-primary hover:bg-primary-hover text-text-bright px-6 py-2.5 rounded font-semibold text-[0.88rem] disabled:opacity-50"
    disabled={applying || unchanged}
    on:click={apply}
  >
    {applying ? stepLabel : $_("language.apply")}
  </button>
  <p class="text-[0.76rem] text-text-muted mt-2">{$_("language.restartNote")}</p>
</div>

{#if errorText}
  <Modal variant="warning" title={$_("common.error")} onClose={() => (errorText = "")}>
    <p class="whitespace-pre-line">{errorText}</p>
  </Modal>
{/if}
