<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from "svelte-i18n";
  import Modal from "../components/Modal.svelte";
  import { api } from "../lib/api";
  import { LOCALE_NAMES, type SupportedLocale } from "../i18n";
  import { REGIONS, applySystemLanguage, type Region } from "../lib/systemLanguage";

  const ORDER: SupportedLocale[] = ["es", "en", "de", "fr", "it", "pt", "ro", "ru"];
  const LAYOUTS: { value: string; label: string }[] = [
    { value: "es", label: "Español" },
    { value: "latam", label: "Español (Latinoamérica)" },
    { value: "us", label: "English (US)" },
    { value: "gb", label: "English (UK)" },
    { value: "ie", label: "English (Ireland)" },
    { value: "de", label: "Deutsch" },
    { value: "fr", label: "Français" },
    { value: "be", label: "Français (Belgique)" },
    { value: "ca", label: "Français (Canada)" },
    { value: "ch", label: "Schweiz / Suisse / Svizzera" },
    { value: "it", label: "Italiano" },
    { value: "pt", label: "Português" },
    { value: "br", label: "Português (Brasil)" },
    { value: "ro", label: "Română" },
    { value: "ru", label: "Русский" },
  ];

  let language: SupportedLocale = "en";
  let region = "en_US.UTF-8";
  let layout = "us";
  let current: SupportedLocale = "en";
  let currentRegion = "en_US.UTF-8";
  let currentLayout = "us";
  // Until the keyboard is chosen by hand it follows the language.
  let layoutTouched = false;
  let applying = false;
  let stepLabel = "";
  let errorText = "";

  onMount(async () => {
    try {
      const state = await api.invoke<{ language: SupportedLocale; locale: string; layout: string }>("get_language_state");
      language = current = state.language;
      region = currentRegion = state.locale;
      layout = currentLayout = state.layout;
    } catch {
      // The defaults stay.
    }
  });

  function pick(code: SupportedLocale) {
    language = code;
    pickRegion(REGIONS[code][0]);
  }

  function pickRegion(r: Region) {
    region = r.locale;
    if (!layoutTouched) layout = r.layout;
  }

  $: unchanged = language === current && region === currentRegion && layout === currentLayout;

  async function apply() {
    applying = true;
    errorText = "";
    try {
      await applySystemLanguage(language, region, layout, (label) => (stepLabel = label));
    } catch (error) {
      errorText = String(error);
      applying = false;
    }
  }
</script>

<div>
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

  <div class="grid grid-cols-2 gap-2 items-start">
    <div>
      {#if REGIONS[language].length > 1}
        <h3 class="text-[0.72rem] font-semibold uppercase tracking-wider text-text-muted mb-2.5">{$_("language.region")}</h3>
        <select
          class="appearance-none w-full bg-surface-2 border border-border rounded px-3 py-2 text-[0.86rem] text-text-bright outline-none focus-visible:ring-2 focus-visible:ring-primary/60 disabled:opacity-60"
          value={region}
          disabled={applying}
          on:change={(e) => pickRegion(REGIONS[language].find((r) => r.locale === e.currentTarget.value) ?? REGIONS[language][0])}
          aria-label={$_("language.region")}
        >
          {#each REGIONS[language] as r}
            <option value={r.locale}>{r.label}</option>
          {/each}
        </select>
      {/if}
    </div>
    <div>
      <h3 class="text-[0.72rem] font-semibold uppercase tracking-wider text-text-muted mb-2.5">{$_("language.keyboard")}</h3>
      <select
        class="appearance-none w-full bg-surface-2 border border-border rounded px-3 py-2 text-[0.86rem] text-text-bright outline-none focus-visible:ring-2 focus-visible:ring-primary/60 disabled:opacity-60"
        bind:value={layout}
        disabled={applying}
        on:change={() => (layoutTouched = true)}
        aria-label={$_("language.keyboard")}
      >
        {#each LAYOUTS as l}
          <option value={l.value}>{l.label}</option>
        {/each}
      </select>
    </div>
  </div>
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
