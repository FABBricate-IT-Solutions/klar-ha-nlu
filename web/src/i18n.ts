import { de } from "./i18n/de";
import { en } from "./i18n/en";
import type { Locale } from "./types";

export type Messages = typeof en;
void (de satisfies Messages);

const RTL = new Set(["ar", "he", "fa", "ur"]);

/** Lazy loaders for the 65 generated chrome packs (U-20). de/en stay in the main chunk. */
const messageLoaders = import.meta.glob("./i18n/messages/*.json") as Record<
  string,
  () => Promise<{ default: Partial<Messages> }>
>;

function packFromPath(path: string): string {
  return path.slice(path.lastIndexOf("/") + 1, -".json".length);
}

const EXTRA_CODES = Object.keys(messageLoaders).map(packFromPath);

/** Every chrome locale Klar ships, including packs not yet fetched. */
export const localeCodes: readonly string[] = Object.freeze(["de", "en", ...EXTRA_CODES]);

const localeCodeSet = new Set<string>(localeCodes);

/** Loaded dictionaries only. Starts with de/en; extras arrive via `ensureLocale`. */
export const dictionaries: Record<string, Messages> = { de, en };

export function isLocale(raw: string | undefined): raw is Locale {
  return Boolean(raw && localeCodeSet.has(raw));
}

export function isRtl(locale: string): boolean {
  return RTL.has(locale);
}

export function matchLocale(raw?: string): Locale | undefined {
  if (!raw) {
    return undefined;
  }
  const tag = raw.replaceAll("_", "-");
  if (localeCodeSet.has(tag)) {
    return tag;
  }
  const lower = tag.toLowerCase();
  for (const code of localeCodes) {
    if (code.toLowerCase() === lower) {
      return code;
    }
  }
  if (lower.startsWith("zh")) {
    if (lower.includes("hk")) {
      return localeCodeSet.has("zh-HK") ? "zh-HK" : undefined;
    }
    if (lower.includes("tw") || lower.includes("hant")) {
      return localeCodeSet.has("zh-TW") ? "zh-TW" : undefined;
    }
    return localeCodeSet.has("zh-CN") ? "zh-CN" : undefined;
  }
  if (lower.startsWith("sr") && lower.includes("latn")) {
    return localeCodeSet.has("sr-Latn") ? "sr-Latn" : undefined;
  }
  const prefix = lower.split("-")[0] || "";
  if (localeCodeSet.has(prefix)) {
    return prefix;
  }
  return undefined;
}

export function chromeLocale(saved?: string): Locale {
  return matchLocale(saved) || "en";
}

/** Fetch and cache a chrome pack. Safe to call repeatedly; falls back to English. */
export async function ensureLocale(code: string): Promise<Messages> {
  const matched = matchLocale(code) || "en";
  const hit = dictionaries[matched];
  if (hit) {
    return hit;
  }
  const path = Object.keys(messageLoaders).find((entry) => packFromPath(entry) === matched);
  if (!path) {
    return dictionaries.en;
  }
  const mod = await messageLoaders[path]!();
  const messages = { ...en, ...mod.default };
  dictionaries[matched] = messages;
  return messages;
}

/** Load every generated pack. Used by tests that assert key parity across locales. */
export async function loadAllLocales(): Promise<void> {
  await Promise.all(EXTRA_CODES.map((code) => ensureLocale(code)));
}

export function assistParseLanguage(languages: string[], chrome?: string): string | undefined {
  return languages.length === 1 ? languages[0] : chrome;
}

export function fill(template: string, slots: Record<string, string>): string {
  return Object.entries(slots).reduce((text, [name, value]) => text.replaceAll(`{${name}}`, value), template);
}

/** Pick a plural form: one / other (and optionally zero). */
export function plural(
  count: number,
  forms: { zero?: string; one: string; other: string },
): string {
  if (count === 0 && forms.zero) {
    return forms.zero;
  }
  if (count === 1) {
    return forms.one;
  }
  return forms.other;
}

const REASON_KEYS = {
  missing_area: "reasonMissingArea",
  weak_name: "reasonWeakName",
  ready: "reasonReady",
} as const;

export function inboxReason(reason: string, t: Messages): string {
  const known = REASON_KEYS[reason as keyof typeof REASON_KEYS];
  if (known) return t[known];
  if (reason.startsWith("match:")) return `${t.reasonMatch}: ${reason.slice("match:".length)}`;
  return reason;
}
