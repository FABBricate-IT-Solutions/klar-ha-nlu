import { describe, expect, it } from "vitest";
import { Pipeline } from "../src/components/pipeline";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { en } from "../src/i18n/en";
import type { ParseResult } from "../src/types";

function result(stages: { stage: string; duration_us: number; detail: string }[]): ParseResult {
  return {
    schema_version: 1,
    text: "turn on the light",
    conversation_id: "c",
    decision: { type: "execute" },
    speech: "ok",
    confidence: 1,
    margin: 0,
    candidates: [],
    plan: { steps: [{ intent: { name: "HassTurnOn", slots: [] } }] },
    evidence: [{ source: "name", value: "light.kugel", score: 1, exact: true }],
    trace: { stages, discarded: [], tokens: ["turn", "on"], normalized: "turn on the light" },
  } as ParseResult;
}

describe("Lab pipeline stage timing (U-17)", () => {
  it("attributes normalize timing to the tokens column, not by array index", () => {
    const html = renderToStaticMarkup(
      createElement(Pipeline, {
        t: en,
        result: result([
          { stage: "normalize", duration_us: 111, detail: "2 tokens" },
          { stage: "ranking", duration_us: 222, detail: "1 ranked" },
          { stage: "action_candidates", duration_us: 333, detail: "1 hyp" },
        ]),
      }),
    );
    expect(html).toContain(`111 ${en.unitsUs}`);
    expect(html).toContain(`333 ${en.unitsUs}`);
    expect(html).toContain(`222 ${en.unitsUs}`);
  });
});
