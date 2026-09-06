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

const STORAGE = "klar_llm_provider";

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
  if (raw.includes(":8080") && raw.endsWith("/v1")) return "llamacpp";
  return "custom";
}

export function readStoredProvider(): LlmProviderId | undefined {
  try {
    const raw = localStorage.getItem(STORAGE) ?? undefined;
    return isProviderId(raw) ? raw : undefined;
  } catch {
    return undefined;
  }
}

export function writeStoredProvider(id: LlmProviderId) {
  try {
    localStorage.setItem(STORAGE, id);
  } catch {
    return;
  }
}

export function resolveProvider(url: string, stored = readStoredProvider()): LlmProviderId {
  const exact = exactProvider(url);
  if (exact !== "custom") return exact;
  if (stored && stored !== "custom") return stored;
  return guessProvider(url);
}
