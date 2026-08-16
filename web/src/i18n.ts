import { de } from "./i18n/de";
import { en } from "./i18n/en";
import type { Locale } from "./types";

export type Messages = typeof de;
const _enHasSameKeys: Messages = en;
export const dictionaries: Record<Locale, Messages> = { de, en };

export function initialLocale(saved?: string, languages: string[] = []): Locale {
  const first = [saved, languages[0], navigator.language].find(Boolean) || "de";
  return first.toLowerCase().startsWith("en") ? "en" : "de";
}
