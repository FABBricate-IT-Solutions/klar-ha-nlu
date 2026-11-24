import { api } from "./api";
import type { LlmProviderId } from "./llmProviders";

const MAX_MODELS = 500;
const MAX_ID = 256;

export function modelAuthHeaders(provider: LlmProviderId, apiKey: string): HeadersInit {
  const headers: Record<string, string> = { Accept: "application/json" };
  const key = apiKey.trim();
  if (!key) return headers;
  if (provider === "anthropic") {
    headers["x-api-key"] = key;
    headers["anthropic-version"] = "2023-06-01";
    return headers;
  }
  headers.Authorization = `Bearer ${key}`;
  return headers;
}

function listingKey(provider: LlmProviderId, apiKey: string): string {
  const key = apiKey.trim();
  if (key) return key;
  if (provider === "lemonade") return "lemonade";
  return "";
}

function cleanIds(value: unknown): string[] {
  if (!Array.isArray(value)) return [];
  const ids = value
    .filter((item): item is string => typeof item === "string")
    .map((item) => item.trim())
    .filter((id) => id.length > 0 && id.length <= MAX_ID && ![...id].some((ch) => ch.charCodeAt(0) < 32));
  return [...new Set(ids)].slice(0, MAX_MODELS);
}

/**
 * The model ids an endpoint offers, asked through the engine. The browser never calls the LLM host
 * itself: a key typed here goes to the engine only, and without one the engine uses the stored key
 * for the stored address and none for any other.
 */
export async function listLlmModels(input: {
  baseUrl: string;
  apiKey: string;
  provider: LlmProviderId;
}): Promise<string[]> {
  const key = listingKey(input.provider, input.apiKey);
  const out = await api.llmModels({
    base_url: input.baseUrl.trim(),
    ...(key ? { api_key: key } : {}),
    provider: input.provider,
  });
  return cleanIds(out.models);
}
