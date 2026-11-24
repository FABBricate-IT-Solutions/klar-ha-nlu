import { screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { App } from "../src/App";
import { LexiconLane } from "../src/components/LexiconLane";
import { LlmSettingsCard } from "../src/components/LlmSettingsCard";
import { en } from "../src/i18n/en";
import { wizardMessages } from "../src/i18n/wizard";
import { CustomPage } from "../src/pages/CustomPage";
import { RoutinesPage } from "../src/pages/RoutinesPage";
import { RulesPage } from "../src/pages/RulesPage";
import { fakeEngine } from "./support/fakeEngine";
import { storedLlm, storedOverlay, storedPolicies, storedSettings, storedUi } from "./support/fixtures";
import { renderUi } from "./support/render";

// Package 1.9: the UI never saves state it did not load. Each test drives an operator flow from
// the bug hunt against an engine that remembers what was posted to it.

const wizard = wizardMessages("en");

function rulesPage() {
  return <RulesPage t={en} locale="en" personality="default" languages={["en"]} rulesView="policies" />;
}

describe("U-01: entering the write token on a token-protected install", () => {
  it("keeps the stored engine settings when the token is typed and Save is pressed", async () => {
    const engine = fakeEngine({ requireToken: "secret123" });
    window.history.replaceState(null, "", "/#/settings/engine");
    const { user } = renderUi(<App />);
    await screen.findByText(/401/);

    await user.type(await screen.findByLabelText(en.token), "secret123");
    const save = screen.getByRole("button", { name: en.save });
    expect(save).toBeDisabled();
    await user.click(save);
    await engine.idle();

    expect(engine.writes()).toEqual([]);
    expect(engine.stored.settings, "engine settings after token and Save").toEqual(storedSettings);
  });

  it("applies the token with its own button, then loads what the engine refused before", async () => {
    const engine = fakeEngine({ requireToken: "secret123" });
    window.history.replaceState(null, "", "/#/settings/engine");
    const { user } = renderUi(<App />);
    await screen.findByText(/401/);
    expect(screen.queryByRole("switch", { name: en.nluRag })).not.toBeInTheDocument();

    await user.type(await screen.findByLabelText(en.token), "secret123");
    await user.click(screen.getByRole("button", { name: en.apply }));

    const toggle = await screen.findByRole("switch", { name: en.nluRag });
    await waitFor(() => expect(screen.queryByText(/401/)).not.toBeInTheDocument());
    expect(localStorage.getItem("klar_token")).toBe("secret123");
    expect(engine.writes("/api/settings")).toEqual([]);

    await user.click(toggle);
    await user.click(screen.getByRole("button", { name: en.save }));

    expect(await screen.findByText(en.saveOk)).toBeInTheDocument();
    expect(engine.stored.settings).toEqual({ ...storedSettings, nlu_rag: true });
    expect(engine.stored.ui.dismissed).toEqual(storedUi.dismissed);
  });

  it("stays on the view the operator opened when the token is applied", async () => {
    fakeEngine({ requireToken: "secret123" });
    const { user } = renderUi(<App />);
    await screen.findByText(/401/);
    await user.click(screen.getByRole("button", { name: en.menu }));
    await user.click(await screen.findByRole("link", { name: en.settings }));
    await user.click(await screen.findByRole("button", { name: en.settingsNavEngine }));

    await user.type(await screen.findByLabelText(en.token), "secret123");
    await user.click(screen.getByRole("button", { name: en.apply }));

    expect(await screen.findByRole("switch", { name: en.nluRag })).toBeInTheDocument();
    expect(window.location.hash).toBe("#/settings/engine");
  });

  it("shows no settings controls for settings it could not read", async () => {
    const engine = fakeEngine();
    engine.failNext("GET /api/settings", 502);
    window.history.replaceState(null, "", "/#/settings/voice");
    renderUi(<App />);
    await screen.findByText(/502/);
    await engine.idle();

    expect(screen.queryAllByRole("switch")).toEqual([]);
    expect(screen.getByRole("button", { name: en.save })).toBeDisabled();
  });
});

describe("U-02: one request fails while the app starts", () => {
  it("does not open the setup wizard over the error", async () => {
    const engine = fakeEngine({ stored: { ui: { ...storedUi, wizard_done: false } } });
    engine.failNext("GET /api/dashboard", 502);
    renderUi(<App />);
    await screen.findByText(/502/);
    await engine.idle();

    expect(screen.queryAllByRole("dialog").length, "wizard dialogs over a failed start").toBe(0);
  });

  it("writes nothing when the operator presses Escape", async () => {
    const engine = fakeEngine();
    engine.failNext("GET /api/dashboard", 502);
    const { user } = renderUi(<App />);
    await screen.findByText(/502/);

    await user.keyboard("{Escape}");
    await engine.idle(500);

    expect(engine.writes("/api/settings"), "settings writes after Escape").toEqual([]);
    expect(engine.stored.settings).toEqual(storedSettings);
    expect(engine.stored.ui).toEqual(storedUi);
  });

  it("writes nothing at all when the UI state itself did not load", async () => {
    const engine = fakeEngine();
    engine.failNext("GET /api/ui", 502);
    const { user } = renderUi(<App />);
    await screen.findByText(/502/);

    await user.keyboard("{Escape}");
    await user.click(screen.getByRole("button", { name: en.menu }));
    await user.click(await screen.findByRole("link", { name: en.lab }));
    await engine.idle(500);

    expect(engine.writes(), "writes without a loaded UI state").toEqual([]);
    expect(engine.stored.ui).toEqual(storedUi);
  });

  it("opens the wizard on a fresh house only when settings, UI state and dashboard all loaded", async () => {
    const engine = fakeEngine({ stored: { ui: { ...storedUi, wizard_done: false } } });
    engine.failNext("GET /api/settings", 502);
    renderUi(<App />);
    await screen.findByText(/502/);
    await engine.idle();

    expect(screen.queryAllByRole("dialog")).toEqual([]);
    expect(engine.writes("/api/settings")).toEqual([]);
  });
});

describe("U-02: leaving the wizard without finishing it", () => {
  it("Skip closes it without saving the settings draft or an LLM endpoint", async () => {
    const engine = fakeEngine({ stored: { ui: { ...storedUi, wizard_done: false } } });
    const { user } = renderUi(<App />);
    const dialog = await screen.findByRole("dialog");
    await user.click(within(dialog).getByRole("button", { name: "3" }));
    await user.type(within(dialog).getByLabelText(en.llmApiKey), "sk-typed-then-skipped");

    await user.click(within(dialog).getByRole("button", { name: wizard.skip }));

    await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
    await engine.idle(500);
    expect(engine.writes("/api/settings")).toEqual([]);
    expect(engine.writes("/api/v2/llm/endpoint")).toEqual([]);
    expect(engine.stored.llmKey).toBe("sk-stored-key");
    // The dismissal itself is remembered, so setup does not come back on every visit.
    expect(engine.stored.ui).toEqual({ ...storedUi, wizard_done: true });
  });

  it("Escape closes it without saving the settings draft or an LLM endpoint", async () => {
    const engine = fakeEngine({ stored: { ui: { ...storedUi, wizard_done: false } } });
    const { user } = renderUi(<App />);
    const dialog = await screen.findByRole("dialog");
    await user.click(within(dialog).getByRole("button", { name: "3" }));
    await user.type(within(dialog).getByLabelText(en.llmApiKey), "sk-typed-then-escaped");

    await user.keyboard("{Escape}");

    await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
    await engine.idle(500);
    expect(engine.writes("/api/settings")).toEqual([]);
    expect(engine.writes("/api/v2/llm/endpoint")).toEqual([]);
    expect(engine.stored.ui).toEqual({ ...storedUi, wizard_done: true });
  });
});

describe("U-03: the house rules fail to load", () => {
  it("sends no rules to the engine when Save is pressed", async () => {
    const engine = fakeEngine();
    engine.failNext("GET /api/v2/policies", 502);
    const { user } = renderUi(rulesPage());
    await screen.findByText(/502/);

    const save = screen.getByRole("button", { name: en.save });
    expect(save).toBeDisabled();
    expect(screen.getByRole("button", { name: en.addRule })).toBeDisabled();
    await user.click(save);
    await engine.idle();

    expect(engine.writes("/api/v2/policies"), "rule writes after a failed load").toEqual([]);
    expect(engine.stored.policies).toEqual(storedPolicies);
  });

  it("does not let a seed switch save an empty rule list", async () => {
    const seed = { id: "seed:confirm-lock", enabled: true, label: "Locks ask first", when: { domain: "lock" }, effect: "confirm" as const };
    const engine = fakeEngine();
    engine.on("GET /api/v2/policies/catalog", { body: { matches: [], seeds: [seed] } });
    engine.failNext("GET /api/v2/policies", 502);
    const { user } = renderUi(rulesPage());
    await screen.findByText("seed:confirm-lock");

    const toggle = screen.getByRole("checkbox", { name: en.seedOn });
    expect(toggle).toBeDisabled();
    await user.click(toggle);
    await engine.idle();

    expect(engine.writes("/api/v2/policies")).toEqual([]);
  });

  it("saves the loaded rules unchanged when nothing failed", async () => {
    const engine = fakeEngine();
    const { user } = renderUi(rulesPage());
    await screen.findByText("No AC at night");

    await user.click(screen.getByRole("button", { name: en.save }));

    await waitFor(() => expect(engine.writes("/api/v2/policies")).toHaveLength(1));
    expect(engine.to("POST /api/v2/policies")[0].body).toEqual(storedPolicies);
  });
});

describe("U-04: adding a lexicon token", () => {
  it("does not send the phrase list, so phrases added in another view survive", async () => {
    const engine = fakeEngine();
    const { user } = renderUi(
      <LexiconLane
        t={en}
        overlay={{ ...storedOverlay, custom: [] }}
        seeds={[]}
        house={[]}
        onSaved={() => undefined}
        onHouse={() => undefined}
        onStatus={() => undefined}
      />,
    );

    // The path field is a combobox (it has a datalist); the token field is the only plain textbox.
    await user.type(screen.getByRole("textbox"), "telly");
    await user.click(screen.getByRole("button", { name: en.lexiconAdd }));

    await waitFor(() => expect(engine.writes("/api/lang/overlay")).toHaveLength(1));
    const body = engine.to("POST /api/lang/overlay")[0].body as Record<string, unknown>;
    expect(body).toMatchObject({ language: { sets: { "nouns.media_nouns": { add: ["telly"] } } } });
    expect(Object.keys(body).includes("custom"), "phrase list in the lexicon save").toBe(false);
    // The lane held a stale, empty copy of the phrase list; the engine still has the phrase.
    expect(engine.stored.overlay.custom).toEqual(storedOverlay.custom);
  });

  it("cannot add or remove a token while the lexicon is not loaded", async () => {
    const engine = fakeEngine();
    engine.failNext("GET /api/lang/overlay", 502);
    const { user } = renderUi(rulesPage());
    await screen.findByText("No AC at night");
    await engine.idle();

    const add = screen.getByRole("button", { name: en.lexiconAdd });
    expect(add).toBeDisabled();
    expect(screen.getByRole("button", { name: en.lexiconRemove })).toBeDisabled();
    await user.click(add);
    await engine.idle();

    expect(engine.writes("/api/lang/overlay")).toEqual([]);
  });

  it("cannot add a phrase while the phrase list is still loading", async () => {
    const engine = fakeEngine();
    engine.on("GET /api/lang/overlay", () => new Promise(() => undefined));
    const { user } = renderUi(<CustomPage t={en} locale="en" embedded />);
    await user.type(screen.getAllByRole("textbox")[0], "film ab bitte");

    const add = screen.getByRole("button", { name: en.addPhrase });
    expect(add).toBeDisabled();
    await user.click(add);
    await engine.idle();

    expect(engine.writes("/api/lang/overlay")).toEqual([]);
    expect(engine.stored.overlay).toEqual(storedOverlay);
  });

  it("cannot add a routine while the phrase list is still loading", async () => {
    const engine = fakeEngine();
    engine.on("GET /api/lang/overlay", () => new Promise(() => undefined));
    const { user } = renderUi(<RoutinesPage t={en} />);

    const add = screen.getByRole("button", { name: en.addRoutine });
    expect(add).toBeDisabled();
    await user.click(add);
    await engine.idle();

    expect(engine.writes("/api/lang/overlay")).toEqual([]);
  });

  it("cannot add a phrase while the phrase list is not loaded", async () => {
    const engine = fakeEngine();
    engine.failNext("GET /api/lang/overlay", 502);
    renderUi(<CustomPage t={en} locale="en" embedded />);
    await screen.findByText(en.engineOffline);

    expect(screen.getByRole("button", { name: en.addPhrase })).toBeDisabled();
    expect(engine.writes("/api/lang/overlay")).toEqual([]);
  });
});

describe("the LLM endpoint fails to load", () => {
  it("cannot be saved over with the empty form", async () => {
    const engine = fakeEngine();
    engine.failNext("GET /api/v2/llm/endpoint", 502);
    const { user } = renderUi(<LlmSettingsCard t={en} />);
    await engine.idle();

    const save = screen.getByRole("button", { name: en.save });
    expect(save).toBeDisabled();
    await user.click(save);
    await engine.idle();

    expect(engine.writes("/api/v2/llm/endpoint")).toEqual([]);
    expect(engine.stored.llm).toEqual(storedLlm);
  });
});

describe("U-08: the setup wizard", () => {
  it("leaves speech refine off when a finished setup is replayed", async () => {
    const engine = fakeEngine({
      stored: {
        settings: { ...storedSettings, refine_speech: false, refine_bands: ["status", "command", "prompt"] },
        ui: { ...storedUi, wizard_done: false },
      },
    });
    const { user } = renderUi(<App />);
    const dialog = await screen.findByRole("dialog");

    await user.click(within(dialog).getByRole("button", { name: "3" }));
    await user.click(within(dialog).getByRole("button", { name: wizard.next }));
    await within(dialog).findByRole("heading", { name: wizard.modeTitle });
    await engine.idle();

    expect(engine.stored.settings.refine_speech, "refine_speech after replaying the LLM step").toBe(false);
    expect(engine.stored.settings.refine_bands).toEqual(["status", "command", "prompt"]);
  });

  it("stays on the step and says why when the engine rejects the save", async () => {
    const engine = fakeEngine({ stored: { ui: { ...storedUi, wizard_done: false } } });
    const { user } = renderUi(<App />);
    const dialog = await screen.findByRole("dialog");
    engine.failNext("POST /api/settings", 401);

    await user.click(within(dialog).getByRole("button", { name: wizard.next }));

    expect(await within(dialog).findByRole("alert")).toHaveTextContent(en.saveUnauthorized);
    expect(within(dialog).getByRole("heading", { level: 1 }).textContent, "wizard step after a rejected save").toBe(wizard.chromeTitle);

    await user.click(within(dialog).getByRole("button", { name: wizard.next }));

    await within(dialog).findByRole("heading", { name: wizard.unitsTitle });
    expect(within(dialog).queryByRole("alert")).not.toBeInTheDocument();
  });

  it("does not finish while the engine rejects the last save", async () => {
    const engine = fakeEngine({ stored: { ui: { ...storedUi, wizard_done: false } } });
    const { user } = renderUi(<App />);
    const dialog = await screen.findByRole("dialog");
    await user.click(within(dialog).getByRole("button", { name: "6" }));
    engine.failNext("POST /api/settings", 500);

    await user.click(within(dialog).getByRole("button", { name: wizard.done }));

    expect(await within(dialog).findByRole("alert")).toHaveTextContent(en.saveFail);
    await engine.idle(500);
    expect(screen.getByRole("dialog")).toBeInTheDocument();
    expect(engine.stored.ui.wizard_done).toBe(false);
  });

  it("does not switch speech refine on when the engine refuses the LLM endpoint", async () => {
    const engine = fakeEngine({
      stored: {
        llm: { configured: false, enable_thinking: false },
        settings: { ...storedSettings, refine_speech: false, refine_bands: [] },
        ui: { ...storedUi, wizard_done: false },
      },
    });
    const { user } = renderUi(<App />);
    const dialog = await screen.findByRole("dialog");
    await user.click(within(dialog).getByRole("button", { name: "3" }));
    await user.type(within(dialog).getByLabelText(en.llmBaseUrl), "http://127.0.0.1:8080/v1");
    engine.failNext("POST /api/v2/llm/endpoint", 422);

    await user.click(within(dialog).getByRole("button", { name: wizard.next }));

    expect(await within(dialog).findByRole("alert")).toHaveTextContent(en.saveFail);
    expect(within(dialog).getByRole("heading", { level: 1 })).toHaveTextContent(wizard.missTitle);
    expect(engine.stored.settings.refine_speech).toBe(false);
    expect(engine.stored.llm.configured).toBe(false);
  });
});
