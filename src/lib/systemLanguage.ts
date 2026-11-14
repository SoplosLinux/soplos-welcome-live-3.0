import { get } from "svelte/store";
import { _ } from "svelte-i18n";
import { api } from "./api";
import { progress } from "./progress";
import type { SupportedLocale } from "../i18n";

// The regions of each language, the first is the default one, and the
// keyboard that goes with each until the user picks another.
export interface Region {
  locale: string;
  label: string;
  layout: string;
}
export const REGIONS: Record<SupportedLocale, Region[]> = {
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

/** The steps that put the system in a language, in order (language_step). */
export const STEPS = ["locale", "user", "xdg", "bookmarks", "restart"] as const;

/**
 * Puts the system in a language: locale, the user's settings, the folders,
 * the bookmarks, and the session restarts. Shared by the Language page and
 * the language switcher of the sidebar, which most people use instead.
 */
export async function applySystemLanguage(
  language: SupportedLocale,
  locale: string,
  layout: string,
  onStep: (label: string) => void = () => {},
): Promise<void> {
  const t = get(_);
  try {
    for (let i = 0; i < STEPS.length; i++) {
      onStep(t(`language.steps.${STEPS[i]}`));
      progress.set(i / STEPS.length);
      await api.invoke("language_step", { step: STEPS[i], lang: language, locale, layout });
    }
    progress.set(1);
  } catch (error) {
    progress.set(null);
    throw error;
  }
}
