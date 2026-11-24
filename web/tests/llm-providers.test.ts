import { describe, expect, it } from "vitest";
import { listLlmModels } from "../src/llmModels";
import { guessProvider, LLM_PROVIDER_IDS, needsKey, providerForUrl, sameBase } from "../src/llmProviders";
import { fakeEngine } from "./support/fakeEngine";

describe("sameBase", () => {
  it("treats case, a default port and trailing slashes as the same address", () => {
    expect(sameBase("https://api.openai.com/v1", "HTTPS://API.OpenAI.com:443/v1/")).toBe(true);
    expect(sameBase("http://llm-box:80/api/v1", "http://llm-box/api/v1")).toBe(true);
  });

  it("reads a URL without a path as /v1, like the engine", () => {
    expect(sameBase("http://127.0.0.1:8080", "http://127.0.0.1:8080/v1")).toBe(true);
    expect(sameBase("http://127.0.0.1:8080/", "http://127.0.0.1:8080/v2")).toBe(false);
  });

  it("tells apart another host, port, scheme or path", () => {
    const stored = "https://api.openai.com/v1";
    expect(sameBase("https://api.openai.com.evil.example/v1", stored)).toBe(false);
    expect(sameBase("https://api.openai.com:8443/v1", stored)).toBe(false);
    expect(sameBase("http://api.openai.com/v1", stored)).toBe(false);
    expect(sameBase("https://api.openai.com/v1/extra", stored)).toBe(false);
  });

  it("never matches something that is not an http URL", () => {
    expect(sameBase("", "")).toBe(false);
    expect(sameBase("api.openai.com/v1", "api.openai.com/v1")).toBe(false);
    expect(sameBase("ftp://host/v1", "ftp://host/v1")).toBe(false);
  });
});

describe("the provider of a URL", () => {
  it("is read from the URL", () => {
    expect(guessProvider("https://api.openai.com/v1")).toBe("openai");
    expect(guessProvider("https://api.anthropic.com/v1/")).toBe("anthropic");
    expect(guessProvider("https://generativelanguage.googleapis.com/v1beta/openai")).toBe("google");
    expect(guessProvider("http://127.0.0.1:13305/api/v1")).toBe("lemonade");
    expect(guessProvider("http://192.168.1.20:8080/v1")).toBe("llamacpp");
    expect(guessProvider("http://llm-box.example.invalid:9000/v1")).toBe("custom");
    expect(guessProvider("")).toBe("custom");
  });

  it("is the saved one only while the URL is the saved address", () => {
    const saved = { base_url: "http://llm-box.example.invalid:9000/v1", provider: "lemonade" };

    expect(providerForUrl("http://llm-box.example.invalid:9000/v1/", saved)).toBe("lemonade");
    expect(providerForUrl("http://llm-box.example.invalid:9001/v1", saved)).toBe("custom");
    expect(providerForUrl("https://api.anthropic.com/v1", saved)).toBe("anthropic");
    expect(providerForUrl("http://llm-box.example.invalid:9000/v1", { ...saved, provider: "made-up" })).toBe("custom");
    expect(providerForUrl("https://api.openai.com/v1")).toBe("openai");
  });

  it("needs a key for the cloud providers only", () => {
    expect(LLM_PROVIDER_IDS.filter(needsKey)).toEqual(["openai", "anthropic", "google"]);
  });
});

describe("listLlmModels", () => {
  it("asks the engine and returns clean, distinct ids", async () => {
    const engine = fakeEngine();
    engine.on("POST /api/v2/llm/models", { body: { models: ["a", " a ", "b", "", 7, "bad\u0007id", null] } });

    await expect(listLlmModels({ baseUrl: " http://127.0.0.1:8080/v1 ", apiKey: "", provider: "llamacpp" })).resolves.toEqual(["a", "b"]);

    expect(engine.to("POST /api/v2/llm/models")[0].body).toEqual({ base_url: "http://127.0.0.1:8080/v1", provider: "llamacpp" });
    expect(engine.external()).toEqual([]);
  });

  it("rejects when the engine refuses, without contacting the LLM host itself", async () => {
    const engine = fakeEngine({ requireToken: "secret123" });

    await expect(listLlmModels({ baseUrl: "http://127.0.0.1:8080/v1", apiKey: "sk-typed", provider: "llamacpp" })).rejects.toThrow("401");

    expect(engine.external()).toEqual([]);
  });

  it("sends the placeholder key Lemonade expects when none was typed", async () => {
    const engine = fakeEngine();

    await listLlmModels({ baseUrl: "http://127.0.0.1:13305/api/v1", apiKey: "", provider: "lemonade" });

    expect(engine.to("POST /api/v2/llm/models")[0].body).toMatchObject({ api_key: "lemonade" });
  });
});
