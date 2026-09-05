import type {
  ApplyRow,
  BundleList,
  ConversationTurn,
  Dashboard,
  Entity,
  EvaluateOut,
  Gaps,
  PolicyBundle,
  PolicyRule,
  MatchCatalog,
  MatchControl,
  LanguageOverlay,
  Settings,
  TrainerContext,
  TrainerProposal,
  TrainerValidateOut,
  UiState,
} from "./types";

export type CustomRule = { phrase: string; intent: string; slots: Record<string, string> };
export type LangOverlay = { custom: CustomRule[]; language: LanguageOverlay; history: Array<{ hash: string; label: string; saved_at: string }> };
export type LangExplain = { language: string; decision: string; confidence: number; speech: string; stages: string[]; evidence: string[]; matched_custom?: string };
export type LanguagePack = { code: string; native_name: string; script: string; variants: string[] };
import { parseV2Response } from "./parseContract";

const jsonHeaders = () => {
  const token = localStorage.getItem("klar_token") || "";
  return {
    "content-type": "application/json",
    ...(token ? { "x-klar-token": token } : {}),
  };
};

const appPath = (path: string) => path.replace(/^\//, "");

async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
  const res = await fetch(appPath(path), { ...init, headers: { ...jsonHeaders(), ...(init.headers || {}) } });
  if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
  return res.json() as Promise<T>;
}

export const api = {
  dashboard: () => request<Dashboard>("/api/dashboard"),
  ui: () => request<UiState>("/api/ui"),
  saveUi: (body: UiState) => request<UiState>("/api/ui", { method: "POST", body: JSON.stringify(body) }),
  settings: () => request<Settings>("/api/settings"),
  saveSettings: (body: Settings) => request<Settings>("/api/settings", { method: "POST", body: JSON.stringify(body) }),
  languages: () => request<LanguagePack[]>("/api/v2/languages"),
  entities: () => request<Entity[]>("/api/entities"),
  gaps: () => request<Gaps>("/api/gaps"),
  custom: () => request<unknown[]>("/api/custom"),
  saveCustom: (body: unknown[]) => request<unknown[]>("/api/custom", { method: "POST", body: JSON.stringify(body) }),
  langOverlay: () => request<LangOverlay>("/api/lang/overlay"),
  saveLangOverlay: (body: { custom: CustomRule[]; language?: unknown; label?: string }) =>
    request<LangOverlay>("/api/lang/overlay", { method: "POST", body: JSON.stringify(body) }),
  previewLang: (body: { text: string; language?: string; custom?: CustomRule[] }) =>
    request<unknown>("/api/lang/preview", { method: "POST", body: JSON.stringify(body) }),
  explainLang: (body: { text: string; language?: string; custom?: CustomRule[] }) =>
    request<LangExplain>("/api/lang/explain", { method: "POST", body: JSON.stringify(body) }),
  rollbackLang: (hash?: string) =>
    request<LangOverlay>("/api/lang/rollback", { method: "POST", body: JSON.stringify({ hash }) }),
  parse: (text: string, language: string, conversation_id?: string, nlu_rag?: boolean, preferred_area?: string) =>
    request<unknown>("/api/v2/parse", { method: "POST", body: JSON.stringify({ text, language, conversation_id, nlu_rag, preferred_area }) }).then(parseV2Response),
  lastTurn: () => request<ConversationTurn | null>("/api/v2/last-turn"),
  policies: () => request<PolicyBundle>("/api/v2/policies"),
  policiesCatalog: () => request<MatchCatalog>("/api/v2/policies/catalog"),
  savePolicies: (body: PolicyBundle) => request<PolicyBundle>("/api/v2/policies", { method: "POST", body: JSON.stringify(body) }),
  evaluatePolicies: (body: { text: string; language?: string; policies?: PolicyRule[]; match_controls?: MatchControl[] }) =>
    request<EvaluateOut>("/api/v2/policies/evaluate", { method: "POST", body: JSON.stringify(body) }),
  trainerContext: (layer: string, language?: string) => {
    const query = new URLSearchParams({ layer });
    if (language) query.set("language", language);
    return request<TrainerContext>(`/api/v2/policies/trainer-context?${query.toString()}`);
  },
  validateProposal: (body: TrainerProposal) =>
    request<TrainerValidateOut>("/api/v2/policies/propose/validate", { method: "POST", body: JSON.stringify(body) }),
  llmEndpoint: () => request<{ configured: boolean; base_url?: string; model?: string }>("/api/v2/llm/endpoint"),
  saveLlmEndpoint: (body: { base_url?: string; api_key?: string; model?: string; configured?: boolean }) =>
    request<{ configured: boolean; base_url?: string; model?: string }>("/api/v2/llm/endpoint", {
      method: "POST",
      body: JSON.stringify(body),
    }),
  conversations: () => request<ConversationTurn[]>("/api/v2/conversations"),
  conversation: (id: string) => request<ConversationTurn[]>(`/api/v2/conversations/${encodeURIComponent(id)}`),
  intents: () => request<string[]>("/api/v2/intents"),
  tagEntity: (body: { entity_id: string; aliases?: string[]; preferred?: boolean; nlu_ignore?: boolean; area?: string }) =>
    request<Entity>("/api/entities", { method: "POST", body: JSON.stringify(body) }),
  bundle: () => request<BundleList>("/api/bundle/entries"),
  deleteBundle: (ids: string[]) => request<BundleList>("/api/bundle/entries", { method: "POST", body: JSON.stringify({ ids }) }),
  clearBundle: () => request<{ enabled: boolean; count: number; bytes: number }>("/api/bundle/clear", { method: "POST" }),
  applySuggestions: () => request<{ applied: number; rows: ApplyRow[] }>("/api/assignment/apply", { method: "POST" }),
  undoApply: () => request<{ applied: number; rows: ApplyRow[] }>("/api/assignment/undo", { method: "POST" }),
};

export function setToken(token: string) {
  if (token.trim()) localStorage.setItem("klar_token", token.trim());
}

export async function download(path: string, fallback: string) {
  const res = await fetch(appPath(path), { headers: jsonHeaders() });
  if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
  const blob = await res.blob();
  const match = (res.headers.get("content-disposition") || "").match(/filename="([^"]+)"/);
  const a = document.createElement("a");
  a.href = URL.createObjectURL(blob);
  a.download = (match && match[1]) || fallback;
  a.click();
  URL.revokeObjectURL(a.href);
}
