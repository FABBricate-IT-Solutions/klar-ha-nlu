export const LLM_PROVIDER_IDS = ["openai", "anthropic", "google", "lemonade", "llamacpp", "custom"] as const;

export type LlmProviderId = (typeof LLM_PROVIDER_IDS)[number];

export type LlmProvider = {
  id: LlmProviderId;
  url: string;
  model: string;
};

export const LLM_PROVIDERS: LlmProvider[] = [
  { id: "openai", url: "https://api.openai.com/v1", model: "gpt-4o-mini" },
  { id: "anthropic", url: "https://api.anthropic.com/v1", model: "claude-sonnet-4-5" },
  { id: "google", url: "https://generativelanguage.googleapis.com/v1beta/openai/", model: "gemini-2.5-flash" },
  { id: "lemonade", url: "http://127.0.0.1:13305/api/v1", model: "" },
  { id: "llamacpp", url: "http://127.0.0.1:8080/v1", model: "" },
  { id: "custom", url: "", model: "" },
];

function norm(url: string): string {
  return url.trim().replace(/\/+$/, "").toLowerCase();
}

export function providerById(id: string): LlmProvider | undefined {
  return LLM_PROVIDERS.find((row) => row.id === id);
}

export function isProviderId(id: string | undefined): id is LlmProviderId {
  return Boolean(id && LLM_PROVIDER_IDS.includes(id as LlmProviderId));
}

export function exactProvider(url: string): LlmProviderId {
  const trimmed = norm(url);
  const hit = LLM_PROVIDERS.find((row) => row.id !== "custom" && norm(row.url) === trimmed);
  return hit?.id ?? "custom";
}

export function guessProvider(url: string): LlmProviderId {
  const exact = exactProvider(url);
  if (exact !== "custom") return exact;
  const raw = norm(url);
  if (!raw) return "custom";
  if (raw.includes("anthropic.com")) return "anthropic";
  if (raw.includes("api.openai.com")) return "openai";
  if (raw.includes("googleapis.com") || raw.includes("generativelanguage")) return "google";
  if (raw.includes("lemonade") || raw.includes(":13305") || raw.includes("/api/v1")) return "lemonade";
  if (raw.includes(":8000") && raw.endsWith("/v1")) return "lemonade";
  if (raw.includes(":8080") && raw.endsWith("/v1")) return "llamacpp";
  return "custom";
}

function baseParts(raw: string): string | undefined {
  try {
    const url = new URL(raw.trim().replace(/\/+$/, ""));
    if (url.protocol !== "http:" && url.protocol !== "https:") return undefined;
    const port = url.port || (url.protocol === "https:" ? "443" : "80");
    // Like the engine: a base URL without a path means `/v1`.
    const path = url.pathname.replace(/\/+$/, "") || "/v1";
    return `${url.protocol}//${url.hostname.toLowerCase()}:${port}${path}`;
  } catch {
    return undefined;
  }
}

/**
 * Whether two base URLs name the same address: scheme, host, port and path. The engine sends the
 * stored API key to the stored address only (`LlmEndpoint::same_base`), so this decides whether an
 * edited URL still gets that key.
 */
export function sameBase(left: string, right: string): boolean {
  const a = baseParts(left);
  return a !== undefined && a === baseParts(right);
}

/**
 * The provider that goes with `url`: the saved one while the URL is still the saved address,
 * otherwise what the URL itself says.
 */
export function providerForUrl(url: string, saved: { base_url?: string; provider?: string } = {}): LlmProviderId {
  if (saved.base_url && isProviderId(saved.provider) && sameBase(url, saved.base_url)) return saved.provider;
  return guessProvider(url);
}

/** Cloud providers refuse a request without a key; local servers and unknown endpoints may run without one. */
export function needsKey(provider: LlmProviderId): boolean {
  switch (provider) {
    case "openai":
    case "anthropic":
    case "google":
      return true;
    case "lemonade":
    case "llamacpp":
    case "custom":
      return false;
    default: {
      const _never: never = provider;
      return _never;
    }
  }
}
