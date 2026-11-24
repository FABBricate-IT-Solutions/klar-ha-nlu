import { vi } from "vitest";
import type { LangOverlay } from "../../src/api";
import type { Dashboard, LlmPublic, PolicyBundle, Settings, UiState } from "../../src/types";
import {
  emptyBundle,
  languagePacks,
  matchCatalog,
  parseResult,
  storedDashboard,
  storedLlm,
  storedOverlay,
  storedPolicies,
  storedSettings,
  storedUi,
} from "./fixtures";

/** One request the UI sent. `path` is the pathname for same-origin calls and origin plus pathname otherwise. */
export type Call = {
  method: string;
  path: string;
  query: string;
  headers: Record<string, string>;
  body: unknown;
  external: boolean;
};

export type Reply = { status?: number; body?: unknown };
export type Handler = Reply | ((call: Call) => Reply);

/** What the engine holds. A POST that the engine accepts changes it; tests read it to see what was clobbered. */
export type Stored = {
  settings: Settings;
  ui: UiState;
  policies: PolicyBundle;
  overlay: LangOverlay;
  llm: LlmPublic;
  /** The LLM key lives in the engine only; the public endpoint view never returns it. */
  llmKey: string;
  dashboard: Dashboard;
};

export type FakeEngine = {
  stored: Stored;
  /** Every request in order, including ones no route answered. */
  calls: Call[];
  /** Same-origin requests the engine had no route for. */
  unhandled: Call[];
  /** Requests that left the engine origin, as `METHOD url`. */
  external: () => string[];
  /** Same-origin requests that write (anything but GET), as `METHOD /path`, optionally for one path. */
  writes: (path?: string) => string[];
  /** Requests for one `METHOD /path` route. */
  to: (route: string) => Call[];
  on: (route: string, handler: Handler) => void;
  /** The next `count` requests to the route answer with `status` and no body. */
  failNext: (route: string, status?: number, count?: number) => void;
  /**
   * Resolves once the UI has stopped talking: no request for `quietMs`. Use it after a click whose
   * correct outcome is "nothing is sent", where there is no element to wait for.
   */
  idle: (quietMs?: number) => Promise<void>;
};

export type FakeEngineOptions = {
  stored?: Partial<Stored>;
  routes?: Record<string, Handler>;
  /** Answer 401 to every request without this `x-klar-token`, as the engine does for LAN peers. */
  requireToken?: string;
};

const STATUS_TEXT: Record<number, string> = {
  401: "Unauthorized",
  404: "Not Found",
  422: "Unprocessable Entity",
  500: "Internal Server Error",
  502: "Bad Gateway",
  503: "Service Unavailable",
};

function clone<T>(value: T): T {
  return structuredClone(value);
}

function routesFor(stored: Stored): Record<string, Handler> {
  return {
    "GET /api/settings": () => ({ body: stored.settings }),
    "POST /api/settings": (call) => {
      stored.settings = call.body as Settings;
      return { body: stored.settings };
    },
    "GET /api/ui": () => ({ body: stored.ui }),
    "POST /api/ui": (call) => {
      stored.ui = call.body as UiState;
      return { body: stored.ui };
    },
    "GET /api/dashboard": () => ({ body: stored.dashboard }),
    "GET /api/v2/conversations": { body: [] },
    "GET /api/v2/last-turn": { body: null },
    "GET /api/v2/languages": { body: languagePacks },
    "GET /api/v2/intents": { body: ["HassTurnOn", "HassTurnOff", "HassGetState"] },
    "GET /api/v2/llm/endpoint": () => ({ body: stored.llm }),
    "POST /api/v2/llm/endpoint": (call) => {
      const { api_key: key, ...rest } = call.body as LlmPublic & { api_key?: string };
      if (rest.configured === false) {
        stored.llm = { configured: false, enable_thinking: false };
        stored.llmKey = "";
      } else {
        stored.llm = { ...stored.llm, ...rest };
        if (key) stored.llmKey = key;
      }
      return { body: stored.llm };
    },
    "POST /api/v2/llm/models": { body: { models: ["house-model", "other-model"] } },
    "POST /api/v2/llm/refine": { body: { type: "done", text: "Refined.", accepted: true } },
    "POST /api/v2/llm/assist": { body: { type: "done", text: "Chat answer." } },
    "GET /api/v2/policies": () => ({ body: stored.policies }),
    "POST /api/v2/policies": (call) => {
      stored.policies = call.body as PolicyBundle;
      return { body: stored.policies };
    },
    "GET /api/v2/policies/catalog": { body: matchCatalog },
    "GET /api/lang/overlay": () => ({ body: stored.overlay }),
    // A body without `custom` leaves the phrase list alone (package 1.9, engine half).
    "POST /api/lang/overlay": (call) => {
      const body = call.body as Partial<LangOverlay>;
      stored.overlay = {
        custom: body.custom ?? stored.overlay.custom,
        language: body.language ?? stored.overlay.language,
        history: stored.overlay.history,
      };
      return { body: stored.overlay };
    },
    "POST /api/v2/parse": (call) => ({ body: parseResult({ text: String((call.body as { text?: string }).text ?? "") }) }),
    "GET /api/bundle/entries": { body: emptyBundle },
    "GET /api/entities": { body: [] },
    "GET /api/gaps": { body: { leftover: [], rooms: [] } },
    "GET /api/custom": { body: [] },
  };
}

function headerMap(init?: RequestInit): Record<string, string> {
  const out: Record<string, string> = {};
  new Headers(init?.headers).forEach((value, name) => {
    out[name.toLowerCase()] = value;
  });
  return out;
}

function parseBody(raw: BodyInit | null | undefined): unknown {
  if (typeof raw !== "string" || !raw) return raw ?? undefined;
  try {
    return JSON.parse(raw) as unknown;
  } catch {
    return raw;
  }
}

function respond(reply: Reply): Response {
  const status = reply.status ?? 200;
  // Through JSON like the wire: the UI never holds a reference into the engine's state.
  const text = status >= 400 && reply.body === undefined ? null : JSON.stringify(reply.body ?? null);
  return new Response(text, {
    status,
    statusText: STATUS_TEXT[status] ?? "",
    headers: { "content-type": "application/json" },
  });
}

/**
 * Replaces `fetch` with an in-memory Klar engine that remembers what was posted to it. Routes are
 * keyed `METHOD /path`. A request to any other origin is recorded as external and answered with
 * 200 `{}`, so a leak shows up as a call instead of a network error.
 */
export function fakeEngine(options: FakeEngineOptions = {}): FakeEngine {
  const stored: Stored = {
    settings: clone(storedSettings),
    ui: clone(storedUi),
    policies: clone(storedPolicies),
    overlay: clone(storedOverlay),
    llm: clone(storedLlm),
    llmKey: "sk-stored-key",
    dashboard: clone(storedDashboard),
    ...clone(options.stored ?? {}),
  };
  const routes: Record<string, Handler> = { ...routesFor(stored), ...(options.routes ?? {}) };
  const failures = new Map<string, { status: number; left: number }>();
  const calls: Call[] = [];
  const unhandled: Call[] = [];
  let lastRequest = Date.now();

  const handle = async (input: RequestInfo | URL, init?: RequestInit): Promise<Response> => {
    const url = new URL(typeof input === "string" || input instanceof URL ? String(input) : input.url, window.location.href);
    const external = url.origin !== window.location.origin;
    const call: Call = {
      method: (init?.method || "GET").toUpperCase(),
      path: external ? `${url.origin}${url.pathname}` : url.pathname,
      query: url.search,
      headers: headerMap(init),
      body: parseBody(init?.body),
      external,
    };
    calls.push(call);
    lastRequest = Date.now();
    if (external) return respond({ body: {} });
    if (options.requireToken && call.headers["x-klar-token"] !== options.requireToken) {
      return respond({ status: 401 });
    }
    const route = `${call.method} ${call.path}`;
    const failure = failures.get(route);
    if (failure && failure.left > 0) {
      failure.left -= 1;
      return respond({ status: failure.status });
    }
    const handler = routes[route];
    if (!handler) {
      unhandled.push(call);
      return respond({ status: 404 });
    }
    return respond(typeof handler === "function" ? handler(call) : handler);
  };

  vi.stubGlobal("fetch", vi.fn(handle));

  const label = (call: Call) => `${call.method} ${call.path}`;
  return {
    stored,
    calls,
    unhandled,
    external: () => calls.filter((call) => call.external).map(label),
    writes: (path) =>
      calls.filter((call) => call.method !== "GET" && !call.external && (!path || call.path === path)).map(label),
    to: (route) => calls.filter((call) => label(call) === route),
    on: (route, handler) => {
      routes[route] = handler;
    },
    failNext: (route, status = 502, count = 1) => {
      failures.set(route, { status, left: count });
    },
    idle: async (quietMs = 150) => {
      lastRequest = Date.now();
      while (Date.now() - lastRequest < quietMs) {
        await new Promise((resolve) => setTimeout(resolve, 10));
      }
    },
  };
}
