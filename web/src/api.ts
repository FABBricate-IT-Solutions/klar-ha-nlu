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
  LlmModels,
  LlmPublic,
  PolicyTrace,
  RefineOutcome,
  Settings,
  LanguageTier,
  TrainerChatEvent,
  TrainerContext,
  TrainerTurn,
  UiState,
  WrongCaseDryRun,
  WrongCaseUpdateDialog,
} from "./types";
import { ApiError, readErrorBody } from "./lib/apiError";
import { takeSseEvents } from "./lib/sse";
import { parseV2Response } from "./parseContract";

export { ApiError, isApiError } from "./lib/apiError";

/** Default client timeout; the HTTP server closes idle heads after 30s. */
export const REQUEST_TIMEOUT_MS = 30_000;

export type CustomRule = { phrase: string; intent: string; slots: Record<string, string> };
export type LangOverlay = { custom: CustomRule[]; language: LanguageOverlay; history: Array<{ hash: string; label: string; saved_at: string }> };
export type LangExplain = {
  language: string;
  decision: string;
  confidence: number;
  speech: string;
  reply?: string;
  stages: string[];
  evidence: string[];
  matched_custom?: string | null;
  policy_trace?: PolicyTrace | null;
};
export type LanguagePack = {
  code: string;
  native_name: string;
  script: string;
  variants: string[];
  tier?: LanguageTier;
  /** False only in an `?all=true` listing: experimental and not selectable yet. */
  offered?: boolean;
};

/** True when the UI is served under Home Assistant ingress (shared HA origin). */
export function isIngressOrigin(): boolean {
  if (typeof window === "undefined") return false;
  const path = `${window.location.pathname}${window.location.search}`;
  return path.includes("hassio_ingress") || path.includes("/api/hassio") || path.includes("supervisor/ingress");
}

/** In-memory write token under ingress so it never lands in HA-origin localStorage (A-10). */
let ingressToken = "";

function readToken(): string {
  if (isIngressOrigin()) return ingressToken;
  return localStorage.getItem("klar_token") || "";
}

const jsonHeaders = () => {
  const token = readToken();
  return {
    "content-type": "application/json",
    ...(token ? { "x-klar-token": token } : {}),
  };
};

/** Resolve `/api/...` under ingress (`.../TOKEN` with no trailing slash) and Vite `/`. */
export function appPath(path: string): string {
  const rel = path.replace(/^\//, "");
  let dir = window.location.pathname;
  if (!dir.endsWith("/")) {
    dir = /\.[a-zA-Z0-9]+$/.test(dir) ? dir.slice(0, dir.lastIndexOf("/") + 1) : `${dir}/`;
  }
  return `${dir}${rel}`;
}

function withTimeout(signal: AbortSignal | null | undefined, ms: number): { signal: AbortSignal; cancel: () => void } {
  const ctrl = new AbortController();
  const timer = window.setTimeout(() => ctrl.abort(new DOMException("Request timed out", "TimeoutError")), ms);
  const onAbort = () => ctrl.abort(signal?.reason);
  signal?.addEventListener("abort", onAbort, { once: true });
  if (signal?.aborted) ctrl.abort(signal.reason);
  return {
    signal: ctrl.signal,
    cancel: () => {
      window.clearTimeout(timer);
      signal?.removeEventListener("abort", onAbort);
    },
  };
}

async function throwHttpError(res: Response): Promise<never> {
  throw new ApiError(res.status, res.statusText, await readErrorBody(res));
}

async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
  const { signal, cancel } = withTimeout(init.signal ?? null, REQUEST_TIMEOUT_MS);
  try {
    const res = await fetch(appPath(path), {
      ...init,
      signal,
      headers: { ...jsonHeaders(), ...(init.headers || {}) },
    });
    if (!res.ok) await throwHttpError(res);
    return (await res.json()) as T;
  } catch (err) {
    if (err instanceof ApiError) throw err;
    if (err instanceof DOMException && (err.name === "AbortError" || err.name === "TimeoutError")) {
      throw new ApiError(0, err.name === "TimeoutError" ? "Request timed out" : "Aborted");
    }
    throw err;
  } finally {
    cancel();
  }
}

async function llmWrite<T>(path: string, body: unknown): Promise<T> {
  try {
    return await request<T>(path, { method: "POST", body: JSON.stringify(body) });
  } catch (err) {
    if (err instanceof ApiError && err.status === 503) {
      throw new Error("llm-unconfigured", { cause: err });
    }
    throw err;
  }
}

function asTrainerChatEvent(raw: unknown): TrainerChatEvent | null {
  if (!raw || typeof raw !== "object") return null;
  const row = raw as { type?: string };
  switch (row.type) {
    case "delta":
    case "done":
    case "error":
    case "proposal":
    case "validate":
    case "consent":
    case "session":
    case "tool_call":
    case "tool":
      return raw as TrainerChatEvent;
    default:
      return null;
  }
}

async function streamTrainerChat(
  body: { message: string; layer?: string; language?: string; history?: TrainerTurn[] },
  onEvent: (event: TrainerChatEvent) => void,
  signal?: AbortSignal,
): Promise<void> {
  const res = await fetch(appPath("/api/v2/policies/trainer/chat"), {
    method: "POST",
    headers: jsonHeaders(),
    body: JSON.stringify(body),
    signal,
  });
  if (res.status === 503) throw new Error("llm-unconfigured");
  if (!res.ok) await throwHttpError(res);
  if (!res.body) return;
  const reader = res.body.getReader();
  const decoder = new TextDecoder();
  let buf = "";
  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    buf += decoder.decode(value, { stream: true });
    const { events, rest } = takeSseEvents(buf);
    buf = rest;
    for (const payload of events) {
      try {
        const event = asTrainerChatEvent(JSON.parse(payload) as unknown);
        if (event) onEvent(event);
      } catch {
        /* skip malformed SSE */
      }
    }
  }
}

export const api = {
  dashboard: () => request<Dashboard>("/api/dashboard"),
  ui: () => request<UiState>("/api/ui"),
  saveUi: (body: UiState) => request<UiState>("/api/ui", { method: "POST", body: JSON.stringify(body) }),
  settings: () => request<Settings>("/api/settings"),
  saveSettings: (body: Settings) => request<Settings>("/api/settings", { method: "POST", body: JSON.stringify(body) }),
  languages: (all = false) =>
    request<LanguagePack[]>(all ? "/api/v2/languages?all=true" : "/api/v2/languages"),
  entities: () => request<Entity[]>("/api/entities"),
  gaps: () => request<Gaps>("/api/gaps"),
  langOverlay: () => request<LangOverlay>("/api/lang/overlay"),
  /** The engine changes only the parts the body names: without `custom` the phrases stay as stored. */
  saveLangOverlay: (body: { custom?: CustomRule[]; language?: unknown; label?: string }) =>
    request<LangOverlay>("/api/lang/overlay", { method: "POST", body: JSON.stringify(body) }),
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
  llmEndpoint: () => request<LlmPublic>("/api/v2/llm/endpoint"),
  customVoice: (body: {
    language: string;
    address: string;
    name?: string;
    voice_name?: string;
    seed?: string;
    warmth: number;
    humor: number;
    sarcasm: number;
    formality: number;
    verbosity: number;
    energy: number;
    taboo?: string;
  }) => request<{ prompt: string }>("/api/v2/llm/custom-voice", { method: "POST", body: JSON.stringify(body) }),
  llmVoice: (personality: string, language: string) => {
    const query = new URLSearchParams({ personality, language });
    return request<{ personality: string; flavor: string; prompt: string }>(`/api/v2/llm/voice?${query.toString()}`);
  },
  saveLlmEndpoint: (body: {
    base_url?: string;
    api_key?: string;
    model?: string;
    configured?: boolean;
    enable_thinking?: boolean;
    provider?: string;
  }) =>
    request<LlmPublic>("/api/v2/llm/endpoint", {
      method: "POST",
      body: JSON.stringify(body),
    }),
  llmModels: (body: { base_url?: string; api_key?: string; provider?: string }) =>
    request<LlmModels>("/api/v2/llm/models", {
      method: "POST",
      body: JSON.stringify(body),
    }),
  llmRefine: (body: {
    speech: string;
    language: string;
    personality: string;
    extra_prompt?: string;
    conversation_id?: string;
  }) =>
    llmWrite<RefineOutcome>("/api/v2/llm/refine", { ...body, stream: false }),
  llmAssist: (body: {
    text: string;
    language: string;
    personality: string;
    extra_prompt?: string;
    conversation_id?: string;
    kind?: string;
  }) =>
    llmWrite<{ type: string; text?: string }>("/api/v2/llm/assist", { ...body, stream: false }).then((row) => ({
      text: String(row.text || ""),
    })),
  trainerChat: (
    body: { message: string; layer?: string; language?: string; history?: TrainerTurn[] },
    onEvent: (event: TrainerChatEvent) => void,
    signal?: AbortSignal,
  ) => streamTrainerChat(body, onEvent, signal),
  trainerConsent: (body: { call_id?: string; decision: "allow_once" | "allow" | "yolo" | "deny" | "ask_again" }) =>
    request<{ ok: boolean; yolo: boolean; allowed: string[] }>("/api/v2/policies/trainer/consent", {
      method: "POST",
      body: JSON.stringify(body),
    }),
  conversations: () => request<ConversationTurn[]>("/api/v2/conversations"),
  markWrong: (body: {
    conversation_id: string;
    ts_ms: number;
    text?: string;
    intent: string;
    slots?: Record<string, string>;
    reject?: boolean;
    clarify?: boolean;
    language?: string;
  }) =>
    request<{ id: string; case_yaml: string }>("/api/v2/conversations/mark-wrong", {
      method: "POST",
      body: JSON.stringify(body),
    }),
  wrongCaseDryRun: () => request<WrongCaseDryRun>("/api/v2/wrong-cases/dry-run"),
  wrongCaseUpdateDialog: () => request<WrongCaseUpdateDialog>("/api/v2/wrong-cases/update-dialog"),
  ackWrongCaseUpdateDialog: () => request<WrongCaseUpdateDialog>("/api/v2/wrong-cases/update-dialog", { method: "POST" }),
  intents: () => request<string[]>("/api/v2/intents"),
  tagEntity: (body: { entity_id: string; aliases?: string[]; preferred?: boolean; nlu_ignore?: boolean; area?: string }) =>
    request<Entity>("/api/entities", { method: "POST", body: JSON.stringify(body) }),
  bundle: () => request<BundleList>("/api/bundle/entries"),
  clearBundle: () => request<{ enabled: boolean; count: number; bytes: number }>("/api/bundle/clear", { method: "POST" }),
  applySuggestions: () => request<{ applied: number; rows: ApplyRow[] }>("/api/assignment/apply", { method: "POST" }),
  undoApply: () => request<{ applied: number; rows: ApplyRow[] }>("/api/assignment/undo", { method: "POST" }),
};

export function setToken(token: string) {
  const next = token.trim();
  if (isIngressOrigin()) {
    ingressToken = next;
    // Drop any leftover LAN token that an older build may have written on this origin.
    localStorage.removeItem("klar_token");
    return;
  }
  if (next) localStorage.setItem("klar_token", next);
  else localStorage.removeItem("klar_token");
}

export function clearToken() {
  ingressToken = "";
  localStorage.removeItem("klar_token");
}

export function getToken(): string {
  return readToken();
}

export async function download(path: string, fallback: string) {
  const { signal, cancel } = withTimeout(null, REQUEST_TIMEOUT_MS);
  try {
    const res = await fetch(appPath(path), { headers: jsonHeaders(), signal });
    if (!res.ok) await throwHttpError(res);
    const blob = await res.blob();
    const match = (res.headers.get("content-disposition") || "").match(/filename="([^"]+)"/);
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = (match && match[1]) || fallback;
    a.click();
    URL.revokeObjectURL(a.href);
  } finally {
    cancel();
  }
}

export async function uploadSettingsBackup(body: Blob) {
  const token = readToken();
  const { signal, cancel } = withTimeout(null, REQUEST_TIMEOUT_MS);
  try {
    const res = await fetch(appPath("/api/v2/settings/restore"), {
      method: "POST",
      signal,
      headers: {
        "content-type": body.type || "application/gzip",
        ...(token ? { "x-klar-token": token } : {}),
      },
      body,
    });
    if (!res.ok) await throwHttpError(res);
  } finally {
    cancel();
  }
}
