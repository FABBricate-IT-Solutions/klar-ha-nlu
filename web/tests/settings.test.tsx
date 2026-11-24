import { screen, waitFor } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { App } from "../src/App";
import { en } from "../src/i18n/en";
import { fakeEngine } from "./support/fakeEngine";
import { storedSettings } from "./support/fixtures";
import { renderUi } from "./support/render";

// Settings as the operator sees them: what a toggle and Save send, and what the result says.

function openSettings(view: string) {
  window.history.replaceState(null, "", `/#/settings/${view}`);
  return renderUi(<App />);
}

describe("the Settings page", () => {
  it("saves a changed toggle together with everything it loaded and says so", async () => {
    const engine = fakeEngine();
    const { user } = openSettings("engine");
    const toggle = await screen.findByRole("switch", { name: en.nluRag });
    expect(toggle).toHaveAttribute("aria-checked", "false");

    await user.click(toggle);
    await user.click(screen.getByRole("button", { name: en.save }));

    expect(await screen.findByText(en.saveOk)).toBeInTheDocument();
    expect(engine.writes("/api/settings")).toEqual(["POST /api/settings"]);
    expect(engine.stored.settings).toEqual({ ...storedSettings, nlu_rag: true });
  });

  it("explains a refused save with the engine's answer", async () => {
    const engine = fakeEngine();
    engine.failNext("POST /api/settings", 500);
    const { user } = openSettings("engine");
    await screen.findByRole("switch", { name: en.nluRag });

    await user.click(screen.getByRole("button", { name: en.save }));

    expect(await screen.findByText(en.saveFail)).toBeInTheDocument();
    expect(screen.getByText("500 Internal Server Error")).toBeInTheDocument();
    expect(engine.stored.settings).toEqual(storedSettings);
  });

  it("points to the write token when a save is unauthorized", async () => {
    const engine = fakeEngine();
    engine.failNext("POST /api/settings", 401);
    const { user } = openSettings("engine");
    await screen.findByRole("switch", { name: en.nluRag });

    await user.click(screen.getByRole("button", { name: en.save }));

    expect(await screen.findByText(en.saveUnauthorized)).toBeInTheDocument();
  });

  it("shows the prompt of the chosen voice, read from the engine", async () => {
    const engine = fakeEngine();
    engine.on("GET /api/v2/llm/voice", { body: { personality: "butler", flavor: "Dry and courteous.", prompt: "You are the house butler." } });
    openSettings("voice");

    expect(await screen.findByText("You are the house butler.")).toBeInTheDocument();
    expect(screen.getByText("Dry and courteous.")).toBeInTheDocument();
    expect(engine.to("GET /api/v2/llm/voice")[0].query).toBe("?personality=butler&language=de");
  });

  it("saves the LLM endpoint from its own card and reports model and URL", async () => {
    const engine = fakeEngine();
    const { user } = openSettings("llm");
    const url = await screen.findByLabelText(en.llmBaseUrl);
    await waitFor(() => expect(url).toHaveValue("https://llm.example.invalid/v1"));

    await user.type(screen.getByLabelText(en.llmApiKey), "sk-rotated");
    // The LLM view has two Save buttons: the page's (engine settings) and, last, the card's own.
    const cardSave = screen.getAllByRole("button", { name: en.save }).at(-1);
    await user.click(cardSave as HTMLElement);

    expect(await screen.findByText(en.saveOk)).toBeInTheDocument();
    expect(engine.to("POST /api/v2/llm/endpoint").at(-1)?.body).toMatchObject({
      base_url: "https://llm.example.invalid/v1",
      model: "house-model",
      api_key: "sk-rotated",
      configured: true,
    });
    expect(engine.stored.llmKey).toBe("sk-rotated");
    expect(engine.writes("/api/settings")).toEqual([]);
  });
});
