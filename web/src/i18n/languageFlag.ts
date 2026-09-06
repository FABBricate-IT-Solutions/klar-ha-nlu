/** ISO language tag → representative ISO 3166-1 alpha-2 for a flag. */
const BASE_COUNTRY: Record<string, string> = {
  af: "ZA",
  ar: "SA",
  bg: "BG",
  bn: "BD",
  ca: "ES",
  cs: "CZ",
  cy: "GB",
  da: "DK",
  de: "DE",
  el: "GR",
  en: "US",
  es: "ES",
  et: "EE",
  eu: "ES",
  fa: "IR",
  fi: "FI",
  fr: "FR",
  ga: "IE",
  gl: "ES",
  gu: "IN",
  he: "IL",
  hi: "IN",
  hr: "HR",
  hu: "HU",
  hy: "AM",
  id: "ID",
  is: "IS",
  it: "IT",
  ja: "JP",
  ka: "GE",
  kn: "IN",
  ko: "KR",
  kw: "GB",
  lb: "LU",
  lt: "LT",
  lv: "LV",
  ml: "IN",
  mn: "MN",
  mr: "IN",
  ms: "MY",
  nb: "NO",
  ne: "NP",
  nl: "NL",
  pa: "IN",
  pl: "PL",
  pt: "PT",
  ro: "RO",
  sk: "SK",
  sl: "SI",
  sr: "RS",
  sv: "SE",
  sw: "KE",
  ta: "IN",
  te: "IN",
  th: "TH",
  tr: "TR",
  uk: "UA",
  ur: "PK",
  vi: "VN",
  zh: "CN",
};

function regionalIndicator(country: string): string {
  return String.fromCodePoint(
    ...[...country.toUpperCase()].map((letter) => 0x1f1e6 + letter.charCodeAt(0) - 65),
  );
}

export function countryFromLanguage(code: string): string | null {
  const parts = code.split(/[-_]/);
  const region = parts.find((part, index) => index > 0 && /^[A-Za-z]{2}$/.test(part));
  if (region) return region.toUpperCase();
  return BASE_COUNTRY[parts[0]?.toLowerCase() ?? ""] ?? null;
}

export function languageFlag(code: string): string {
  const country = countryFromLanguage(code);
  return country ? regionalIndicator(country) : "🌐";
}
