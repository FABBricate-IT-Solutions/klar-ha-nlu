import { api } from "./api";
import type { LlmProviderId } from "./llmProviders";

const MAX_MODELS = 500;
const MAX_ID = 256;

export function modelListUrls(baseUrl: string): string[] {
  const base = baseUrl.trim().replace(/\/+$/, "");
  if (!base) return [];
  const urls = [`${base}/models`];
  if (base.endsWith("/api/v1")) {
    urls.push(`${base.slice(0, -"/api/v1".length)}/v1/models`);
  } else if (base.endsWith("/v1")) {
    urls.push(`${base.slice(0, -"/v1".length)}/api/v1/models`);
  }
  return [...new Set(urls)];
}

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

export function parseModelIds(value: unknown): string[] {
  const rows = modelRows(value);
  const parsed: { id: string; labels: string[] }[] = [];
  for (const row of rows) {
    const id = modelId(row);
    if (!id) continue;
    parsed.push({ id, labels: rowLabels(row) });
  }
  const chat = parsed.filter((row) => row.labels.includes("chat")).map((row) => row.id);
  const ids = chat.length ? chat : parsed.map((row) => row.id);
  return [...new Set(ids)].slice(0, MAX_MODELS);
}

function modelRows(value: unknown): unknown[] {
  if (Array.isArray(value)) return value;
  if (value && typeof value === "object") {
    const rec = value as Record<string, unknown>;
    if (Array.isArray(rec.data)) return rec.data;
    if (Array.isArray(rec.models)) return rec.models;
  }
  return [];
}

function modelId(row: unknown): string | undefined {
  if (typeof row === "string") return sanitizeId(row);
  if (!row || typeof row !== "object") return undefined;
  const rec = row as Record<string, unknown>;
  for (const key of ["id", "name", "model"]) {
    const value = rec[key];
    if (typeof value === "string") return sanitizeId(value);
  }
  return undefined;
}

function rowLabels(row: unknown): string[] {
  if (!row || typeof row !== "object") return [];
  const labels = (row as { labels?: unknown }).labels;
  if (!Array.isArray(labels)) return [];
  return labels.filter((item): item is string => typeof item === "string").map((item) => item.toLowerCase());
}

function sanitizeId(raw: string): string | undefined {
  const id = raw.trim();
  if (!id || id.length > MAX_ID || [...id].some((ch) => ch.charCodeAt(0) < 32)) return undefined;
  return id;
}

function listingKey(provider: LlmProviderId, apiKey: string): string {
  const key = apiKey.trim();
  if (key) return key;
  if (provider === "lemonade") return "lemonade";
  return "";
}

async function fetchModelsDirect(baseUrl: string, apiKey: string, provider: LlmProviderId): Promise<string[]> {
  const headers = modelAuthHeaders(provider, apiKey);
  let lastEmpty = false;
  for (const url of modelListUrls(baseUrl)) {
    try {
      const response = await fetch(url, { headers });
      if (!response.ok) continue;
      const ids = parseModelIds(await response.json());
      if (ids.length) return ids;
      lastEmpty = true;
    } catch {
      continue;
    }
  }
  if (lastEmpty) return [];
  throw new Error("models");
}

export async function listLlmModels(input: {
  baseUrl: string;
  apiKey: string;
  provider: LlmProviderId;
}): Promise<string[]> {
  const baseUrl = input.baseUrl.trim();
  const key = listingKey(input.provider, input.apiKey);
  try {
    const out = await api.llmModels({
      base_url: baseUrl,
      ...(key ? { api_key: key } : {}),
      provider: input.provider,
    });
    if (out.models.length) return out.models;
    return out.models;
  } catch {
    return fetchModelsDirect(baseUrl, key, input.provider);
  }
}
