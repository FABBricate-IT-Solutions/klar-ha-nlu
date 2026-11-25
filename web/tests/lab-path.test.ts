import { describe, expect, it } from "vitest";
import { en } from "../src/i18n/en";
import { armedPipeline, labChatLike, labPath, labRefineEligible, refineBandOf } from "../src/pages/ParsePage";
import { parseV2Response } from "../src/parseContract";
import type { Settings } from "../src/types";
import { parseResult, storedSettings } from "./support/fixtures";

// The Lab shows which stages one sentence went through. `labPath` derives that from the parse
// answer and the settings; these cases pin the path for each kind of answer.

const plain: Settings = {
  ...storedSettings,
  personality: "default",
  refine_speech: false,
  refine_bands: [],
  quiet_ack: false,
  calendar_llm: false,
  allow_llm_tools: false,
  nlu_rag: false,
  semantic_adapters: false,
};

describe("the parse fixture", () => {
  it("has the wire shape the UI accepts from the engine", () => {
    expect(() => parseV2Response(structuredClone(parseResult()))).not.toThrow();
    expect(() => parseV2Response(structuredClone(parseResult({ decision: { type: "reject", reason: "no_target" } })))).not.toThrow();
    expect(() => parseV2Response(structuredClone(parseResult({ decision: { type: "chat" }, speech: "" })))).not.toThrow();
  });
});

describe("labPath", () => {
  it("is the parse step and a placeholder before anything was parsed", () => {
    expect(labPath(null, plain, en, "en")).toEqual([`${en.labParse} · en`, "…"]);
    expect(labPath(null, plain, en)).toEqual([en.labParse, "…"]);
  });

  it("names the intent of an executed command", () => {
    expect(labPath(parseResult(), plain, en, "en")).toEqual([`${en.labParse} · en`, "HassTurnOn"]);
  });

  it("joins the intents of a plan with several steps", () => {
    const result = parseResult({ intents: ["HassTurnOn", "HassLightSet"] });

    expect(labPath(result, plain, en)).toEqual([en.labParse, "HassTurnOn · HassLightSet"]);
  });

  it("adds the LLM refine step only for a band that is switched on", () => {
    const command = parseResult({ slots: { entity_id: "cover.garage_door" } });
    const refineCommands: Settings = { ...plain, refine_speech: true, refine_bands: ["command"] };
    const refineStatus: Settings = { ...plain, refine_speech: true, refine_bands: ["status"] };

    expect(labPath(command, refineCommands, en)).toEqual([en.labParse, "HassTurnOn", en.labChipLlmRefine]);
    expect(labPath(command, refineStatus, en)).toEqual([en.labParse, "HassTurnOn"]);
  });

  it("shows a quiet acknowledgement instead of refine for a plain light switch", () => {
    const settings: Settings = { ...plain, quiet_ack: true, refine_speech: true, refine_bands: ["command"] };

    expect(labPath(parseResult(), settings, en)).toEqual([en.labParse, "HassTurnOn", en.labChipQuietAck]);
  });

  it("routes a calendar question through the calendar LLM when that is on", () => {
    const result = parseResult({ intents: ["KlarGetCalendarEvents"], slots: {} });

    expect(labPath(result, { ...plain, calendar_llm: true }, en)).toEqual([en.labParse, "KlarGetCalendarEvents", en.labChipCalendarLlm]);
    expect(labPath(result, plain, en)).toEqual([en.labParse, "KlarGetCalendarEvents"]);
  });

  it("marks a chat answer as LLM chat and never refines it", () => {
    const chat = parseResult({ decision: { type: "chat" }, speech: "" });
    const settings: Settings = { ...plain, refine_speech: true, refine_bands: ["status", "command", "prompt", "reject"] };

    expect(labPath(chat, settings, en)).toEqual([en.labParse, "chat", en.labChipLlmChat]);
    expect(labPath(chat, { ...settings, allow_llm_tools: true }, en)).toEqual([en.labParse, "chat", en.labChipLlmChat, en.labChipLlmTools]);
  });

  it("shows retrieval and semantic adapters only where the engine uses them", () => {
    const reject = parseResult({ decision: { type: "reject", reason: "no_target" }, speech: "I did not get that." });
    const settings: Settings = { ...plain, nlu_rag: true, semantic_adapters: true };

    expect(labPath(reject, settings, en)).toEqual([en.labParse, en.labChipNluRag, en.labChipSemantic, "reject"]);
    expect(labPath(parseResult(), settings, en)).toEqual([en.labParse, "HassTurnOn"]);
  });

  it("names the confirmation step for a risky command", () => {
    const confirm = parseResult({ decision: { type: "confirm", prompt: "Unlock the front door?", candidate_id: "plan-000" } });

    expect(labPath(confirm, plain, en)).toEqual([en.labParse, "confirm", en.labChipConfirmRisky]);
    expect(labPath(confirm, { ...plain, confirm_risky_actions: false }, en)).toEqual([en.labParse, "confirm"]);
  });

  it("labels a house rule that hands the sentence to the LLM", () => {
    const result = parseResult({ decision: { type: "reject", reason: "policy" }, policy_trace: { hit: "llm" } });

    expect(labPath(result, plain, en)).toEqual([en.labParse, en.effectLlm, en.labChipLlmChat]);
  });
});

describe("armedPipeline", () => {
  it("is empty for an engine without extras", () => {
    expect(armedPipeline(plain, en)).toEqual([]);
  });

  it("lists every switched-on extra in pipeline order", () => {
    const settings: Settings = {
      ...plain,
      personality: "butler",
      mode: "context_only",
      nlu_rag: true,
      semantic_adapters: true,
      confirm_risky_actions: false,
      refine_speech: true,
      calendar_llm: true,
      quiet_ack: true,
      allow_llm_tools: true,
    };

    expect(armedPipeline(settings, en)).toEqual([
      "butler",
      en.labChipContextOnly,
      en.labChipNluRag,
      en.labChipSemantic,
      en.labChipNoConfirm,
      en.labChipLlmRefine,
      en.labChipCalendarLlm,
      en.labChipQuietAck,
      en.labChipLlmTools,
    ]);
  });
});

describe("the refine band of an answer", () => {
  it("is status for a question and command for an action", () => {
    expect(refineBandOf(parseResult({ intents: ["HassGetState"] }))).toBe("status");
    expect(refineBandOf(parseResult())).toBe("command");
  });

  it("is the band the engine reports when it reports one", () => {
    expect(refineBandOf(parseResult({ refine_band: "status" }))).toBe("status");
  });

  it("is prompt for a question back, reject for a refusal, and none for chat", () => {
    expect(refineBandOf(parseResult({ decision: { type: "clarify", prompt: "Which lamp?", options: ["Desk", "Ceiling"] } }))).toBe("prompt");
    expect(refineBandOf(parseResult({ decision: { type: "reject", reason: "no_target" } }))).toBe("reject");
    expect(refineBandOf(parseResult({ decision: { type: "chat" } }))).toBeNull();
    expect(refineBandOf(null)).toBeNull();
  });

  it("decides whether the Lab asks the LLM to refine", () => {
    const status = parseResult({ intents: ["HassGetState"], speech: "The hallway lamp is on." });

    expect(labRefineEligible(status, { ...plain, refine_speech: true, refine_bands: ["status"] })).toBe(true);
    expect(labRefineEligible(status, { ...plain, refine_speech: true, refine_bands: ["command"] })).toBe(false);
    expect(labRefineEligible(status, plain)).toBe(false);
    expect(labRefineEligible(parseResult({ speech: "  " }), { ...plain, refine_speech: true, refine_bands: ["command"] })).toBe(false);
    expect(labChatLike(parseResult({ briefing: true }))).toBe(true);
  });
});
