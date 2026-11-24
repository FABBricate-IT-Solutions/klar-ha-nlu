import { screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { App } from "../src/App";
import { en } from "../src/i18n/en";
import { wizardMessages } from "../src/i18n/wizard";
import { fakeEngine, type FakeEngineOptions } from "./support/fakeEngine";
import { storedSettings, storedUi } from "./support/fixtures";
import { renderUi, settle } from "./support/render";

// What each wizard step writes to the engine, seen from the requests the engine receives.

const copy = wizardMessages("en");

async function openWizard(options: FakeEngineOptions["stored"] = {}) {
  const engine = fakeEngine({ stored: { ui: { ...storedUi, wizard_done: false }, ...options } });
  const view = renderUi(<App />);
  const dialog = await screen.findByRole("dialog");
  return { engine, dialog, user: view.user };
}

describe("the setup wizard", () => {
  it("opens on a house that has not finished setup and names its first step", async () => {
    const { dialog } = await openWizard();

    expect(within(dialog).getByRole("heading", { level: 1 })).toHaveTextContent(copy.chromeTitle);
    expect(within(dialog).getByRole("button", { name: "1" })).toHaveAttribute("aria-current", "step");
  });

  it("stays closed on a house that finished setup", async () => {
    fakeEngine();
    renderUi(<App />);
    await screen.findByRole("heading", { name: en.understandsHome });

    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("saves the engine settings when a settings step is left with Next", async () => {
    const { engine, dialog, user } = await openWizard();

    await user.click(within(dialog).getByRole("button", { name: copy.next }));
    await within(dialog).findByRole("heading", { name: copy.unitsTitle });

    expect(engine.writes("/api/settings")).toEqual(["POST /api/settings"]);
    expect(engine.stored.settings).toEqual(storedSettings);
  });

  it("stores the LLM endpoint and turns speech refine on for status answers on first setup", async () => {
    const { engine, dialog, user } = await openWizard({
      llm: { configured: false, enable_thinking: false },
      settings: { ...storedSettings, refine_speech: false, refine_bands: [] },
    });

    await user.click(within(dialog).getByRole("button", { name: "3" }));
    await user.type(within(dialog).getByLabelText(en.llmBaseUrl), "http://127.0.0.1:8080/v1");
    await user.type(within(dialog).getByLabelText(en.llmApiKey), "sk-first");
    await user.click(within(dialog).getByRole("button", { name: copy.next }));
    await within(dialog).findByRole("heading", { name: copy.modeTitle });
    await settle(20);

    const endpoint = engine.to("POST /api/v2/llm/endpoint").at(-1)?.body;
    expect(endpoint).toMatchObject({ base_url: "http://127.0.0.1:8080/v1", api_key: "sk-first", configured: true, provider: "llamacpp" });
    expect(engine.stored.llm).toMatchObject({ configured: true, base_url: "http://127.0.0.1:8080/v1" });
    expect(engine.stored.settings.refine_speech).toBe(true);
    expect(engine.stored.settings.refine_bands).toEqual(["status"]);
  });

  it("leaves the LLM endpoint alone when the LLM step is left empty", async () => {
    const { engine, dialog, user } = await openWizard({
      llm: { configured: false, enable_thinking: false },
      settings: { ...storedSettings, refine_speech: false, refine_bands: [] },
    });

    await user.click(within(dialog).getByRole("button", { name: "3" }));
    await user.click(within(dialog).getByRole("button", { name: copy.next }));
    await within(dialog).findByRole("heading", { name: copy.modeTitle });

    expect(engine.writes("/api/v2/llm/endpoint")).toEqual([]);
    expect(engine.stored.settings.refine_speech).toBe(false);
  });

  it("marks setup as finished in the UI state when Done is pressed", async () => {
    const { engine, dialog, user } = await openWizard();

    await user.click(within(dialog).getByRole("button", { name: "6" }));
    await user.click(within(dialog).getByRole("button", { name: copy.done }));

    await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
    expect(engine.stored.ui.wizard_done).toBe(true);
    expect(engine.stored.ui.locale).toBe("en");
  });

  it("speaks the saved UI language", async () => {
    const german = wizardMessages("de");
    const { dialog } = await openWizard({ ui: { ...storedUi, wizard_done: false, locale: "de" } });

    expect(within(dialog).getByRole("heading", { level: 1 })).toHaveTextContent(german.chromeTitle);
    expect(within(dialog).getByRole("button", { name: german.next })).toBeInTheDocument();
  });

  it("can be reopened from Settings", async () => {
    fakeEngine();
    window.history.replaceState(null, "", "/#/settings/engine");
    const { user } = renderUi(<App />);

    await user.click(await screen.findByRole("button", { name: en.setupReplay }));

    expect(await screen.findByRole("dialog")).toBeInTheDocument();
  });
});
