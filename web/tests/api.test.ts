import { describe, expect, it } from "vitest";
import { ApiError, api, appPath, setToken } from "../src/api";
import { takeSseEvents } from "../src/lib/sse";
import { fakeEngine } from "./support/fakeEngine";
import { parseResult, storedOverlay, storedSettings, storedUi } from "./support/fixtures";

// The API client: where requests go under ingress, what they carry, and how failures surface.

describe("appPath", () => {
  it("resolves under the page directory, so the UI works behind Home Assistant ingress", () => {
    window.history.replaceState(null, "", "/api/hassio_ingress/abc123");
    expect(appPath("/api/settings")).toBe("/api/hassio_ingress/abc123/api/settings");

    window.history.replaceState(null, "", "/api/hassio_ingress/abc123/");
    expect(appPath("/api/settings")).toBe("/api/hassio_ingress/abc123/api/settings");
  });

  it("resolves next to a served file and at the root", () => {
    window.history.replaceState(null, "", "/ui/index.html");
    expect(appPath("/api/settings")).toBe("/ui/api/settings");

    window.history.replaceState(null, "", "/");
    expect(appPath("api/settings")).toBe("/api/settings");
  });
});

describe("the API client", () => {
  it("sends JSON and no token header when no token is stored", async () => {
    const engine = fakeEngine();

    await expect(api.settings()).resolves.toEqual(storedSettings);

    expect(engine.calls[0].headers).toEqual({ "content-type": "application/json" });
  });

  it("sends the stored write token with every request, trimmed", async () => {
    const engine = fakeEngine({ requireToken: "secret123" });
    await expect(api.settings()).rejects.toThrow("401 Unauthorized");

    setToken("  secret123  ");

    await expect(api.saveSettings({ ...storedSettings, quiet_ack: false })).resolves.toMatchObject({ quiet_ack: false });
    expect(engine.calls.at(-1)?.headers["x-klar-token"]).toBe("secret123");
    expect(engine.stored.settings.quiet_ack).toBe(false);
  });

  it("keeps the stored token when a blank one is applied", () => {
    setToken("secret123");

    setToken("   ");

    expect(localStorage.getItem("klar_token")).toBe("secret123");
  });

  it("rejects with status and reason when the engine refuses", async () => {
    const engine = fakeEngine();
    engine.failNext("POST /api/v2/policies", 422);

    await expect(api.savePolicies({ policies: [], speech_bank: { entries: [] } })).rejects.toThrow("422 Unprocessable Entity");
  });

  it("surfaces the engine error text and path from a JSON body (U-18)", async () => {
    const engine = fakeEngine();
    engine.failNext("POST /api/v2/policies", 500, 1, { error: "could not save: disk full", path: "klar_nlu.json" });

    await expect(api.savePolicies({ policies: [], speech_bank: { entries: [] } })).rejects.toMatchObject({
      name: "ApiError",
      message: "could not save: disk full",
      path: "klar_nlu.json",
      status: 500,
    } satisfies Partial<ApiError>);
  });

  it("reads every data line in an SSE event, not only the first (U-28)", () => {
    const { events, rest } = takeSseEvents('data: {"type":"delta","text":"a"}\ndata: {"type":"done"}\n\npartial');
    expect(events).toEqual(['{"type":"delta","text":"a"}\n{"type":"done"}']);
    expect(rest).toBe("partial");
  });

  it("posts each save to its own endpoint with the body it was given", async () => {
    const engine = fakeEngine();

    await api.saveLlmEndpoint({ base_url: "http://127.0.0.1:8080/v1", model: "m", api_key: "sk-new", configured: true });
    await api.saveUi({ ...storedUi, theme: "light" });
    await api.saveLangOverlay({ custom: storedOverlay.custom, label: "test" });

    expect(engine.writes()).toEqual(["POST /api/v2/llm/endpoint", "POST /api/ui", "POST /api/lang/overlay"]);
    expect(engine.to("POST /api/v2/llm/endpoint")[0].body).toEqual({ base_url: "http://127.0.0.1:8080/v1", model: "m", api_key: "sk-new", configured: true });
    expect(engine.stored.llmKey).toBe("sk-new");
    expect(engine.stored.ui.theme).toBe("light");
    expect(engine.to("POST /api/lang/overlay")[0].body).toEqual({ custom: storedOverlay.custom, label: "test" });
  });

  it("puts trainer context filters in the query", async () => {
    const engine = fakeEngine();
    engine.on("GET /api/v2/policies/trainer-context", { body: {} });

    await api.trainerContext("house", "de");

    expect(engine.calls[0].query).toBe("?layer=house&language=de");
  });

  it("names a missing LLM so callers can tell it from a failure", async () => {
    const engine = fakeEngine();
    engine.on("POST /api/v2/llm/refine", { status: 503 });
    engine.on("POST /api/v2/llm/assist", { status: 500 });

    await expect(api.llmRefine({ speech: "ok", language: "en", personality: "default" })).rejects.toThrow("llm-unconfigured");
    await expect(api.llmAssist({ text: "hi", language: "en", personality: "default" })).rejects.toThrow("500 Internal Server Error");
  });

  it("validates a parse answer against the v2 contract", async () => {
    const engine = fakeEngine();
    await expect(api.parse("turn on the hallway lamp", "en")).resolves.toMatchObject({ decision: { type: "execute" } });

    engine.on("POST /api/v2/parse", { body: { ...parseResult(), plan: undefined } });

    await expect(api.parse("turn on the hallway lamp", "en")).rejects.toThrow("plan must be an object");
  });
});
