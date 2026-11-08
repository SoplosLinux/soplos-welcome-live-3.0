<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from "svelte-i18n";
  import Modal from "../components/Modal.svelte";
  import { api } from "../lib/api";
  import { progress } from "../lib/progress";
  import { LOCALE_NAMES, type SupportedLocale } from "../i18n";

  // The regions of each language, the first is the default one, and the
  // keyboard that goes with each until the user picks another.
  interface Region {
    locale: string;
    label: string;
    layout: string;
  }
  const REGIONS: Record<SupportedLocale, Region[]> = {
    es: [
      { locale: "es_ES.UTF-8", label: "España", layout: "es" },
      { locale: "es_MX.UTF-8", label: "México", layout: "latam" },
      { locale: "es_AR.UTF-8", label: "Argentina", layout: "latam" },
      { locale: "es_CO.UTF-8", label: "Colombia", layout: "latam" },
      { locale: "es_CL.UTF-8", label: "Chile", layout: "latam" },
      { locale: "es_PE.UTF-8", label: "Perú", layout: "latam" },
      { locale: "es_VE.UTF-8", label: "Venezuela", layout: "latam" },
      { locale: "es_EC.UTF-8", label: "Ecuador", layout: "latam" },
      { locale: "es_UY.UTF-8", label: "Uruguay", layout: "latam" },
      { locale: "es_BO.UTF-8", label: "Bolivia", layout: "latam" },
      { locale: "es_PY.UTF-8", label: "Paraguay", layout: "latam" },
      { locale: "es_CR.UTF-8", label: "Costa Rica", layout: "latam" },
      { locale: "es_GT.UTF-8", label: "Guatemala", layout: "latam" },
      { locale: "es_PA.UTF-8", label: "Panamá", layout: "latam" },
      { locale: "es_DO.UTF-8", label: "República Dominicana", layout: "latam" },
      { locale: "es_HN.UTF-8", label: "Honduras", layout: "latam" },
      { locale: "es_NI.UTF-8", label: "Nicaragua", layout: "latam" },
      { locale: "es_SV.UTF-8", label: "El Salvador", layout: "latam" },
      { locale: "es_CU.UTF-8", label: "Cuba", layout: "latam" },
      { locale: "es_PR.UTF-8", label: "Puerto Rico", layout: "latam" },
    ],
    en: [
      { locale: "en_US.UTF-8", label: "United States", layout: "us" },
      { locale: "en_GB.UTF-8", label: "United Kingdom", layout: "gb" },
      { locale: "en_AU.UTF-8", label: "Australia", layout: "us" },
      { locale: "en_CA.UTF-8", label: "Canada", layout: "us" },
      { locale: "en_IE.UTF-8", label: "Ireland", layout: "ie" },
      { locale: "en_NZ.UTF-8", label: "New Zealand", layout: "us" },
      { locale: "en_ZA.UTF-8", label: "South Africa", layout: "us" },
      { locale: "en_IN.UTF-8", label: "India", layout: "us" },
    ],
    de: [
      { locale: "de_DE.UTF-8", label: "Deutschland", layout: "de" },
      { locale: "de_AT.UTF-8", label: "Österreich", layout: "de" },
      { locale: "de_CH.UTF-8", label: "Schweiz", layout: "ch" },
    ],
    fr: [
      { locale: "fr_FR.UTF-8", label: "France", layout: "fr" },
      { locale: "fr_BE.UTF-8", label: "Belgique", layout: "be" },
      { locale: "fr_CH.UTF-8", label: "Suisse", layout: "ch" },
      { locale: "fr_CA.UTF-8", label: "Canada", layout: "ca" },
    ],
    it: [
      { locale: "it_IT.UTF-8", label: "Italia", layout: "it" },
      { locale: "it_CH.UTF-8", label: "Svizzera", layout: "ch" },
    ],
    pt: [
      { locale: "pt_PT.UTF-8", label: "Portugal", layout: "pt" },
      { locale: "pt_BR.UTF-8", label: "Brasil", layout: "br" },
    ],
    ro: [
      { locale: "ro_RO.UTF-8", label: "România", layout: "ro" },
    ],
    ru: [
      { locale: "ru_RU.UTF-8", label: "Россия", layout: "ru" },
    ],
  };
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
    const steps = ["locale", "user", "xdg", "bookmarks", "restart"];
    try {
      for (let i = 0; i < steps.length; i++) {
        stepLabel = $_(`language.steps.${steps[i]}`);
        progress.set(i / steps.length);
        await api.invoke("language_step", { step: steps[i], lang: language, locale: region, layout });
      }
      progress.set(1);
    } catch (error) {
      errorText = String(error);
      progress.set(null);
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
