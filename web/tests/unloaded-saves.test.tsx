import { screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { App } from "../src/App";
import { LexiconLane } from "../src/components/LexiconLane";
import { en } from "../src/i18n/en";
import { wizardMessages } from "../src/i18n/wizard";
import { RulesPage } from "../src/pages/RulesPage";
import { fakeEngine } from "./support/fakeEngine";
import { storedOverlay, storedPolicies, storedSettings, storedUi } from "./support/fixtures";
import { knownBug } from "./support/knownBug";
import { renderUi, settle, type User } from "./support/render";

// Package 1.9: the UI must never save state it did not load. Each test drives the operator flow
// from the bug hunt against an engine that remembers what was posted to it.

const wizard = wizardMessages("en");

/** The wizard is the only dialog the app can show before the operator opens one. */
function wizardDialogs(): HTMLElement[] {
  return screen.queryAllByRole("dialog");
}

/** Today a failed start opens the wizard (U-02). The operator gets past it the way the UI offers. */
async function skipWizardIfOpen(user: User) {
  const [dialog] = wizardDialogs();
  if (!dialog) return;
  await user.click(within(dialog).getByRole("button", { name: wizard.skip }));
  await waitFor(() => expect(wizardDialogs()).toHaveLength(0));
}

function rulesPage() {
  return <RulesPage t={en} locale="en" personality="default" languages={["en"]} rulesView="policies" />;
}

describe("U-01: entering the write token on a token-protected install", () => {
  knownBug("U-01", "keeps the stored engine settings when the token is typed and Save is pressed", /^engine settings after token and Save/, async () => {
    const engine = fakeEngine({ requireToken: "secret123" });
    window.history.replaceState(null, "", "/#/settings/engine");
    const { user } = renderUi(<App />);
    await screen.findByText(/401/);
    await skipWizardIfOpen(user);

    await user.type(await screen.findByLabelText(en.token), "secret123");
    await user.click(screen.getByRole("button", { name: en.save }));
    await settle(50);

    expect(engine.stored.settings, "engine settings after token and Save").toEqual(storedSettings);
  });
});

describe("U-02: one request fails while the app starts", () => {
  knownBug("U-02", "does not open the setup wizard over the error", /^wizard dialogs over a failed start/, async () => {
    const engine = fakeEngine();
    engine.failNext("GET /api/dashboard", 502);
    renderUi(<App />);
    await screen.findByText(/502/);

    expect(wizardDialogs().length, "wizard dialogs over a failed start").toBe(0);
  });

  knownBug("U-02", "writes nothing when the operator presses Escape", /^writes after Escape/, async () => {
    const engine = fakeEngine();
    engine.failNext("GET /api/dashboard", 502);
    const { user } = renderUi(<App />);
    await screen.findByText(/502/);

    await user.keyboard("{Escape}");
    await settle(50);

    expect(engine.writes(), "writes after Escape").toEqual([]);
    expect(engine.stored.settings).toEqual(storedSettings);
    expect(engine.stored.ui).toEqual(storedUi);
  });

  knownBug("U-02", "writes nothing when the operator presses Skip", /^writes after Skip/, async () => {
    const engine = fakeEngine();
    engine.failNext("GET /api/settings", 502);
    const { user } = renderUi(<App />);
    await screen.findByText(/502/);

    await skipWizardIfOpen(user);
    await settle(50);

    expect(engine.writes(), "writes after Skip").toEqual([]);
    expect(engine.stored.settings).toEqual(storedSettings);
    expect(engine.stored.ui).toEqual(storedUi);
  });
});

describe("U-03: the house rules fail to load", () => {
  knownBug("U-03", "sends no rules to the engine when Save is pressed", /^rule writes after a failed load/, async () => {
    const engine = fakeEngine();
    engine.failNext("GET /api/v2/policies", 502);
    const { user } = renderUi(rulesPage());
    await screen.findByText(/502/);

    await user.click(screen.getByRole("button", { name: en.save }));
    await settle(50);

    expect(engine.writes("/api/v2/policies"), "rule writes after a failed load").toEqual([]);
    expect(engine.stored.policies).toEqual(storedPolicies);
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
  knownBug("U-04", "does not send the phrase list, so phrases added in another view survive", /^phrase list in the lexicon save/, async () => {
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
});

describe("U-08: the setup wizard", () => {
  knownBug("U-08", "leaves speech refine off when a finished setup is replayed", /^refine_speech after replaying the LLM step/, async () => {
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
    await settle(50);

    expect(engine.stored.settings.refine_speech, "refine_speech after replaying the LLM step").toBe(false);
    expect(engine.stored.settings.refine_bands).toEqual(["status", "command", "prompt"]);
  });

  knownBug("U-08", "stays on the step when the engine rejects the save", /^wizard step after a rejected save/, async () => {
    const engine = fakeEngine({ stored: { ui: { ...storedUi, wizard_done: false } } });
    const { user } = renderUi(<App />);
    const dialog = await screen.findByRole("dialog");
    engine.failNext("POST /api/settings", 401);

    await user.click(within(dialog).getByRole("button", { name: wizard.next }));
    await settle(50);

    expect(within(dialog).getByRole("heading", { level: 1 }).textContent, "wizard step after a rejected save").toBe(wizard.chromeTitle);
  });
});
