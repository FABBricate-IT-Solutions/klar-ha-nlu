import { de } from "./i18n/de";
import { en } from "./i18n/en";
import type { Locale } from "./types";

export type Messages = typeof en;
void (de satisfies Messages);

const RTL = new Set(["ar", "he", "fa", "ur"]);

const extras = import.meta.glob("./i18n/messages/*.json", { eager: true, import: "default" }) as Record<
  string,
  Partial<Messages>
>;

function packFromPath(path: string): string {
  return path.slice(path.lastIndexOf("/") + 1, -".json".length);
}

export const dictionaries: Record<string, Messages> = { de, en };
for (const [path, extra] of Object.entries(extras)) {
  dictionaries[packFromPath(path)] = { ...en, ...extra };
}

export function isLocale(raw: string | undefined): raw is Locale {
  return Boolean(raw && dictionaries[raw]);
}

export function isRtl(locale: string): boolean {
  return RTL.has(locale);
}

export function matchLocale(raw?: string): Locale | undefined {
  if (!raw) {
    return undefined;
  }
  const tag = raw.replaceAll("_", "-");
  if (dictionaries[tag]) {
    return tag;
  }
  const lower = tag.toLowerCase();
  for (const code of Object.keys(dictionaries)) {
    if (code.toLowerCase() === lower) {
      return code;
    }
  }
  if (lower.startsWith("zh")) {
    if (lower.includes("hk")) {
      return dictionaries["zh-HK"] ? "zh-HK" : undefined;
    }
    if (lower.includes("tw") || lower.includes("hant")) {
      return dictionaries["zh-TW"] ? "zh-TW" : undefined;
    }
    return dictionaries["zh-CN"] ? "zh-CN" : undefined;
  }
  if (lower.startsWith("sr") && lower.includes("latn")) {
    return dictionaries["sr-Latn"] ? "sr-Latn" : undefined;
  }
  const prefix = lower.split("-")[0] || "";
  if (dictionaries[prefix]) {
    return prefix;
  }
  return undefined;
}

function browserLanguage(): string {
  return typeof navigator !== "undefined" ? navigator.language : "";
}

export function chromeLocale(languages: string[] = [], saved?: string): Locale {
  if (languages.length === 1) {
    return matchLocale(languages[0]) || matchLocale(saved) || matchLocale(browserLanguage()) || "de";
  }
  return matchLocale(saved) || matchLocale(browserLanguage()) || "de";
}

export function fill(template: string, slots: Record<string, string>): string {
  return Object.entries(slots).reduce((text, [name, value]) => text.replaceAll(`{${name}}`, value), template);
}
