import { screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { App } from "../src/App";
import { LlmSettingsCard } from "../src/components/LlmSettingsCard";
import { en } from "../src/i18n/en";
import { wizardMessages } from "../src/i18n/wizard";
import type { LlmPublic } from "../src/types";
import { fakeEngine, type FakeEngine } from "./support/fakeEngine";
import { storedLlm, storedUi } from "./support/fixtures";
import { renderUi, settle, type User } from "./support/render";

// Package 1.8, UI half: the stored LLM key belongs to the stored address. The model list is the
// request that could carry a key to whatever the URL field names, so the UI asks for it only when
// the operator opens the list, only through the engine, and never as a side effect of typing.

const LOAD = "GET /api/v2/llm/endpoint";
const MODELS = "POST /api/v2/llm/models";
const openAi: LlmPublic = { configured: true, base_url: "https://api.openai.com/v1", model: "gpt-4o-mini", enable_thinking: false, provider: "openai" };

async function card(engine: FakeEngine, stored: LlmPublic = storedLlm) {
  const view = renderUi(<LlmSettingsCard t={en} />);
  const url = await screen.findByLabelText(en.llmBaseUrl);
  await waitFor(() => expect(url).toHaveValue(stored.base_url));
  await engine.idle();
  return { user: view.user, url };
}

/** Everything the UI sent except the one request that fills the form. */
function caused(engine: FakeEngine): string[] {
  return engine.calls.map((call) => `${call.method} ${call.path}`).filter((route) => route !== LOAD);
}

async function retype(user: User, url: HTMLElement, value: string) {
  await user.clear(url);
  await user.type(url, value);
}

describe("A-01: the model list", () => {
  it("is not requested when the settings open", async () => {
    const engine = fakeEngine();
    await card(engine);
    await settle(600);

    expect(caused(engine)).toEqual([]);
  });

  it("is not requested while the URL is typed", async () => {
    const engine = fakeEngine();
    const { user, url } = await card(engine);

    await user.clear(url);
    await user.type(url, "http://203.0.113.7:8080");
    await settle(600);
    await user.type(url, "/v1");
    await settle(600);

    expect(caused(engine), "requests caused by typing the URL").toEqual([]);
  });

  it("is not requested while a key is typed", async () => {
    const engine = fakeEngine();
    const { user } = await card(engine);

    await user.type(screen.getByLabelText(en.llmApiKey), "sk-typed");
    await settle(600);

    expect(caused(engine)).toEqual([]);
  });

  it("is asked from the engine once when the operator opens the list", async () => {
    const engine = fakeEngine();
    const { user } = await card(engine);

    await user.click(screen.getByLabelText(en.llmModel));

    expect(await screen.findByRole("option", { name: "other-model" })).toBeInTheDocument();
    expect(caused(engine)).toEqual([MODELS]);
    // No key was typed, so none is sent: the engine decides whether the stored key applies.
    expect(engine.to(MODELS)[0].body).toEqual({ base_url: storedLlm.base_url, provider: "custom" });

    await user.keyboard("{Escape}");
    await user.click(screen.getByLabelText(en.llmModel));
    await engine.idle();

    expect(caused(engine)).toEqual([MODELS]);
  });

  it("sends a key the operator typed for this address along with the request", async () => {
    const engine = fakeEngine();
    const { user, url } = await card(engine);
    await retype(user, url, "https://api.anthropic.com/v1");
    await user.type(screen.getByLabelText(en.llmApiKey), "sk-ant-typed");

    await user.click(screen.getByLabelText(en.llmModel));
    await engine.idle();

    expect(engine.to(MODELS).map((call) => call.body)).toEqual([
      { base_url: "https://api.anthropic.com/v1", api_key: "sk-ant-typed", provider: "anthropic" },
    ]);
  });

  it("never falls back to a request from the browser when the engine cannot list models", async () => {
    const engine = fakeEngine();
    engine.on(MODELS, { status: 502 });
    const { user } = await card(engine);

    await user.click(screen.getByLabelText(en.llmModel));

    expect(await screen.findByText(en.llmModelsFail)).toBeInTheDocument();
    await engine.idle();
    expect(engine.external(), "requests from the browser to the LLM host").toEqual([]);
    expect(caused(engine)).toEqual([MODELS]);
  });

  it("tries again when the list is opened after a failure", async () => {
    const engine = fakeEngine();
    engine.failNext(MODELS, 502);
    const { user } = await card(engine);
    await user.click(screen.getByLabelText(en.llmModel));
    await screen.findByText(en.llmModelsFail);

    await user.keyboard("{Escape}");
    await user.click(screen.getByLabelText(en.llmModel));

    expect(await screen.findByRole("option", { name: "other-model" })).toBeInTheDocument();
    expect(screen.queryByText(en.llmModelsFail)).not.toBeInTheDocument();
  });

  it("drops a loaded list when the URL changes and asks nothing until the list is opened again", async () => {
    const engine = fakeEngine();
    const { user, url } = await card(engine);
    await user.click(screen.getByLabelText(en.llmModel));
    await screen.findByRole("option", { name: "other-model" });
    await user.keyboard("{Escape}");

    await retype(user, url, "http://127.0.0.1:8080/v1");
    await settle(600);

    expect(caused(engine)).toEqual([MODELS]);
    await user.click(screen.getByLabelText(en.llmModel));
    await waitFor(() => expect(engine.to(MODELS)).toHaveLength(2));
    expect(engine.to(MODELS)[1].body).toEqual({ base_url: "http://127.0.0.1:8080/v1", provider: "llamacpp" });
  });

  it("says when the endpoint offers no models", async () => {
    const engine = fakeEngine();
    engine.on(MODELS, { body: { models: [] } });
    const { user } = await card(engine);

    await user.click(screen.getByLabelText(en.llmModel));

    expect(await screen.findByText(en.llmModelsEmpty)).toBeInTheDocument();
  });
});

describe("A-01: the provider follows the URL", () => {
  it("is derived again when the URL changes, and that provider is saved", async () => {
    const engine = fakeEngine({ stored: { llm: { ...openAi, base_url: "https://api.anthropic.com/v1", provider: "anthropic" } } });
    const { user, url } = await card(engine, { ...openAi, base_url: "https://api.anthropic.com/v1" });
    const provider = screen.getAllByRole("combobox")[0];
    expect(provider).toHaveValue(en.llmPresetAnthropic);

    // Not a preset URL: only the URL itself can say that this is no longer Anthropic.
    await retype(user, url, "http://192.168.1.20:8080/v1");

    expect(provider).toHaveValue(en.llmPresetLlamaCpp);
    await user.click(screen.getByRole("button", { name: en.save }));
    await waitFor(() => expect(engine.writes("/api/v2/llm/endpoint")).toHaveLength(1));
    expect(engine.to("POST /api/v2/llm/endpoint")[0].body).toMatchObject({ base_url: "http://192.168.1.20:8080/v1", provider: "llamacpp" });
  });

  it("stays the saved one while the URL is the saved address", async () => {
    const stored: LlmPublic = { configured: true, base_url: "http://llm-box.example.invalid:9000/v1", model: "m", provider: "lemonade" };
    const engine = fakeEngine({ stored: { llm: stored } });
    const { user, url } = await card(engine, stored);
    const provider = screen.getAllByRole("combobox")[0];
    expect(provider).toHaveValue(en.llmPresetLemonade);

    await user.type(url, "/");

    expect(provider).toHaveValue(en.llmPresetLemonade);
    await user.click(screen.getByRole("button", { name: en.save }));
    await waitFor(() => expect(engine.writes("/api/v2/llm/endpoint")).toHaveLength(1));
    expect(engine.to("POST /api/v2/llm/endpoint")[0].body).toMatchObject({ provider: "lemonade" });
  });
});

describe("A-01: the key belongs to the stored address", () => {
  it("is demanded before a cloud endpoint at another address can be saved", async () => {
    const engine = fakeEngine({ stored: { llm: openAi } });
    const { user, url } = await card(engine, openAi);
    const key = screen.getByLabelText(en.llmApiKey);
    const save = screen.getByRole("button", { name: en.save });
    expect(key).toHaveAttribute("placeholder", en.llmApiKeyHint);
    expect(save).toBeEnabled();

    await retype(user, url, "https://api.anthropic.com/v1");

    expect(key).toBeRequired();
    expect(key).toBeInvalid();
    expect(key).not.toHaveAttribute("placeholder", en.llmApiKeyHint);
    expect(save).toBeDisabled();
    await user.click(save);
    await engine.idle();
    expect(engine.writes("/api/v2/llm/endpoint")).toEqual([]);

    await user.type(key, "sk-ant-new");

    expect(key).toBeValid();
    await user.click(save);
    await waitFor(() => expect(engine.writes("/api/v2/llm/endpoint")).toHaveLength(1));
    expect(engine.to("POST /api/v2/llm/endpoint")[0].body).toMatchObject({ base_url: "https://api.anthropic.com/v1", api_key: "sk-ant-new", provider: "anthropic" });
    expect(engine.stored.llmKey).toBe("sk-ant-new");
  });

  it("is demanded for another path on the same host as well, as the engine drops it there too", async () => {
    const engine = fakeEngine({ stored: { llm: openAi } });
    const { user, url } = await card(engine, openAi);

    await retype(user, url, "https://api.openai.com/v2");

    expect(screen.getByLabelText(en.llmApiKey)).toBeRequired();
    expect(screen.getByRole("button", { name: en.save })).toBeDisabled();
  });

  it("is kept, and not asked for, while the URL still names the stored address", async () => {
    const engine = fakeEngine({ stored: { llm: openAi } });
    const { user, url } = await card(engine, openAi);

    await retype(user, url, "HTTPS://API.OpenAI.com:443/v1/");

    const key = screen.getByLabelText(en.llmApiKey);
    expect(key).not.toBeRequired();
    expect(key).toHaveAttribute("placeholder", en.llmApiKeyHint);
    expect(screen.getByRole("button", { name: en.save })).toBeEnabled();
  });

  it("is not demanded for a local server, but the field stops promising the stored key", async () => {
    const engine = fakeEngine({ stored: { llm: openAi } });
    const { user, url } = await card(engine, openAi);

    await retype(user, url, "http://127.0.0.1:8080/v1");

    const key = screen.getByLabelText(en.llmApiKey);
    expect(key).not.toBeRequired();
    expect(key).not.toHaveAttribute("placeholder", en.llmApiKeyHint);
    await user.click(screen.getByRole("button", { name: en.save }));
    await waitFor(() => expect(engine.writes("/api/v2/llm/endpoint")).toHaveLength(1));
    expect(engine.to("POST /api/v2/llm/endpoint")[0].body).not.toHaveProperty("api_key");
  });

  it("is demanded in the setup wizard too: Next does not store a cloud endpoint without its key", async () => {
    const copy = wizardMessages("en");
    const engine = fakeEngine({ stored: { llm: openAi, ui: { ...storedUi, wizard_done: false } } });
    const { user } = renderUi(<App />);
    const dialog = await screen.findByRole("dialog");
    await user.click(within(dialog).getByRole("button", { name: "3" }));
    const url = within(dialog).getByLabelText(en.llmBaseUrl);
    await waitFor(() => expect(url).toHaveValue(openAi.base_url));

    await retype(user, url, "https://api.anthropic.com/v1");
    await settle(600);
    await user.click(within(dialog).getByRole("button", { name: copy.next }));

    expect(await within(dialog).findByRole("alert")).toHaveTextContent(en.saveFail);
    expect(within(dialog).getByLabelText(en.llmApiKey)).toBeInvalid();
    expect(within(dialog).getByRole("heading", { level: 1 })).toHaveTextContent(copy.missTitle);
    expect(engine.writes("/api/v2/llm/endpoint")).toEqual([]);
    expect(engine.writes("/api/v2/llm/models")).toEqual([]);
    expect(engine.stored.llm).toEqual(openAi);
  });
});
