import { de } from "./i18n/de";
import { en } from "./i18n/en";
import type { Locale } from "./types";

export type Messages = typeof de;
const _enHasSameKeys: Messages = en;
export const dictionaries: Record<Locale, Messages> = { de, en };

export function initialLocale(saved?: string, languages: string[] = []): Locale {
  const first = (languages[0] || "").toLowerCase();
  if (first.startsWith("en")) {
    return "en";
  }
  if (saved === "en" || saved === "de") {
    return saved;
  }
  const nav = typeof navigator !== "undefined" ? navigator.language : "";
  return (nav || "de").toLowerCase().startsWith("en") ? "en" : "de";
}
