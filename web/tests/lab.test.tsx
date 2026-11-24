import { screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { en } from "../src/i18n/en";
import { ParsePage } from "../src/pages/ParsePage";
import type { Settings } from "../src/types";
import { fakeEngine } from "./support/fakeEngine";
import { parseResult, storedSettings } from "./support/fixtures";
import { renderUi } from "./support/render";

// The Lab page: one sentence goes to the engine's parse endpoint, and the answer decides whether
// the LLM is asked to refine it, to chat, or not at all.

const quiet: Settings = { ...storedSettings, personality: "default", refine_speech: false, refine_bands: [], quiet_ack: false };
const rooms = [{ area_id: "hallway", name: "Hallway" }];

function lab(settings: Settings = quiet, parseLanguage: string | undefined = "en") {
  return <ParsePage t={en} parseLanguage={parseLanguage} replayText="" settings={settings} rooms={rooms} />;
}

async function typeSentence(user: ReturnType<typeof renderUi>["user"], text: string) {
  const box = screen.getByLabelText(en.command);
  await user.clear(box);
  await user.type(box, text);
}

describe("the Lab page", () => {
  it("sends the sentence to the parse endpoint and shows decision, intent and path", async () => {
    const engine = fakeEngine();
    const { user } = renderUi(lab());
    await typeSentence(user, "turn on the hallway lamp");

    await user.click(screen.getByRole("button", { name: en.analyze }));

    expect(await screen.findByText("The hallway lamp is on.")).toBeInTheDocument();
    expect(engine.to("POST /api/v2/parse")).toHaveLength(1);
    expect(engine.to("POST /api/v2/parse")[0].body).toEqual({ text: "turn on the hallway lamp", language: "en" });
    expect(screen.getAllByText("HassTurnOn", { selector: "strong" }).length).toBeGreaterThan(0);
    expect(screen.getByText("entity_id: light.hallway_lamp")).toBeInTheDocument();
    const path = within(screen.getByLabelText(en.labThisTurn)).getAllByText(/.+/).map((chip) => chip.textContent);
    expect(path).toEqual([`${en.labParse} · en`, "HassTurnOn", en.refineBandCommand]);
    expect(engine.writes("/api/v2/llm/refine")).toEqual([]);
    expect(engine.writes("/api/v2/llm/assist")).toEqual([]);
  });

  it("starts with the sample sentence and parses on Enter", async () => {
    const engine = fakeEngine();
    const { user } = renderUi(lab());
    const box = screen.getByLabelText(en.command);
    expect(box).toHaveValue(en.parseSample);

    await user.click(box);
    await user.keyboard("{Enter}");

    await waitFor(() => expect(engine.to("POST /api/v2/parse")).toHaveLength(1));
    expect(engine.to("POST /api/v2/parse")[0].body).toMatchObject({ text: en.parseSample });
  });

  it("keeps the conversation and passes the room the sentence was heard in", async () => {
    const engine = fakeEngine();
    const { user } = renderUi(lab({ ...quiet, nlu_rag: true }));

    await user.click(screen.getByRole("button", { name: en.analyze }));
    await screen.findByText("The hallway lamp is on.");
    await user.click(screen.getByRole("button", { name: en.analyze }));

    await waitFor(() => expect(engine.to("POST /api/v2/parse")).toHaveLength(2));
    const [first, second] = engine.to("POST /api/v2/parse").map((call) => call.body as Record<string, unknown>);
    expect(first.conversation_id).toBeUndefined();
    expect(second.conversation_id).toBe("conv-1");
    expect(second.nlu_rag).toBe(true);
  });

  it("asks the LLM to refine an answer whose band is switched on", async () => {
    const engine = fakeEngine();
    const { user } = renderUi(lab({ ...quiet, personality: "butler", refine_speech: true, refine_bands: ["command"] }));

    await user.click(screen.getByRole("button", { name: en.analyze }));

    expect(await screen.findByText("Refined.")).toBeInTheDocument();
    expect(engine.to("POST /api/v2/llm/refine")[0].body).toMatchObject({
      speech: "The hallway lamp is on.",
      language: "en",
      personality: "butler",
      conversation_id: "conv-1",
      stream: false,
    });
    expect(within(screen.getByLabelText(en.labThisTurn)).getByText(en.labChipLlmRefine)).toBeInTheDocument();
  });

  it("asks the LLM to answer a sentence the engine hands to chat", async () => {
    const engine = fakeEngine();
    engine.on("POST /api/v2/parse", { body: parseResult({ decision: { type: "chat" }, speech: "", text: "tell me a joke" }) });
    const { user } = renderUi(lab());
    await typeSentence(user, "tell me a joke");

    await user.click(screen.getByRole("button", { name: en.analyze }));

    expect(await screen.findByText("Chat answer.")).toBeInTheDocument();
    expect(engine.to("POST /api/v2/llm/assist")[0].body).toMatchObject({ text: "tell me a joke", language: "en", stream: false });
    expect(engine.writes("/api/v2/llm/refine")).toEqual([]);
  });

  it("says that no LLM is configured when the engine answers 503", async () => {
    const engine = fakeEngine();
    engine.on("POST /api/v2/parse", { body: parseResult({ decision: { type: "chat" }, speech: "" }) });
    engine.on("POST /api/v2/llm/assist", { status: 503 });
    const { user } = renderUi(lab());

    await user.click(screen.getByRole("button", { name: en.analyze }));

    expect(await screen.findByText(en.llmNotConfigured)).toBeInTheDocument();
  });

  it("shows the engine's error instead of a result when the parse fails", async () => {
    const engine = fakeEngine();
    engine.failNext("POST /api/v2/parse", 500);
    const { user } = renderUi(lab());

    await user.click(screen.getByRole("button", { name: en.analyze }));

    expect(await screen.findByRole("alert")).toHaveTextContent("500 Internal Server Error");
    expect(screen.queryByLabelText(en.labThisTurn)).not.toBeInTheDocument();
  });

  it("refuses an answer that does not match the parse contract", async () => {
    const engine = fakeEngine();
    engine.on("POST /api/v2/parse", { body: { ...parseResult(), schema_version: "1.0" } });
    const { user } = renderUi(lab());

    await user.click(screen.getByRole("button", { name: en.analyze }));

    expect(await screen.findByRole("alert")).toHaveTextContent("Unsupported parse schema");
  });

  it("lists the switched-on extras as the armed pipeline", () => {
    fakeEngine();
    renderUi(lab({ ...quiet, personality: "butler", quiet_ack: true }));

    const armed = within(screen.getByLabelText(en.labPipeline)).getAllByText(/.+/).map((chip) => chip.textContent);
    expect(armed).toEqual(["butler", en.labChipQuietAck]);
  });
});
