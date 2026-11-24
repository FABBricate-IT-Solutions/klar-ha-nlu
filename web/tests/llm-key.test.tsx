import { screen, waitFor } from "@testing-library/react";
import { describe, expect } from "vitest";
import { LlmSettingsCard } from "../src/components/LlmSettingsCard";
import { en } from "../src/i18n/en";
import { fakeEngine } from "./support/fakeEngine";
import { storedLlm } from "./support/fixtures";
import { knownBug } from "./support/knownBug";
import { renderUi, settle } from "./support/render";

// Package 1.8, UI half: the stored LLM key belongs to the stored host. The engine attaches the
// stored key to a model-list request when the UI sends none, so every model-list request for a URL
// that is still being typed hands the key to whatever host the half-typed URL names.

const LOAD = "GET /api/v2/llm/endpoint";

describe("A-01: editing the LLM URL", () => {
  knownBug("A-01", "sends no request while the URL is typed", /^requests caused by typing the URL/, async () => {
    const engine = fakeEngine();
    const { user } = renderUi(<LlmSettingsCard t={en} />);
    const url = await screen.findByLabelText(en.llmBaseUrl);
    await waitFor(() => expect(url).toHaveValue(storedLlm.base_url));

    await user.clear(url);
    await user.type(url, "http://203.0.113.7:8080");
    await settle(600);
    await user.type(url, "/v1");
    await settle(600);

    const caused = engine.calls.map((call) => `${call.method} ${call.path}`).filter((route) => route !== LOAD);
    expect(caused, "requests caused by typing the URL").toEqual([]);
  });
});
