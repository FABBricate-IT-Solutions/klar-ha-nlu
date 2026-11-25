import fs from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { upsertMatchControl } from "../src/components/MatchLane";
import { bakeVariants, pruneSpeechBank } from "../src/speechBank";
import type { MatchCatalogRow, MatchControl, PolicyRule } from "../src/types";

describe("match precedence field (U-24)", () => {
  const catalog: MatchCatalogRow[] = [{ id: "timer", precedence: 0, summary_key: "timer" }];

  it("clears an override when the operator empties the field", () => {
    const rows: MatchControl[] = [{ id: "timer", enabled: true, precedence: 7 }];
    expect(upsertMatchControl(rows, catalog, "timer", { precedence: undefined })).toEqual([]);
  });

  it("keeps a typed non-default precedence", () => {
    expect(upsertMatchControl([], catalog, "timer", { precedence: 7 })).toEqual([
      { id: "timer", enabled: true, precedence: 7 },
    ]);
  });
});

describe("speech bank bake and prune (U-34)", () => {
  it("bakes English lines for non-de locales instead of skipping them", () => {
    const entry = bakeVariants("rule-1", "confirm", "default", ["fr", "nl", "ja"]);
    expect(entry.variants.length).toBeGreaterThan(0);
    expect(entry.variants.every((row) => ["fr", "nl", "ja"].includes(row.language))).toBe(true);
    expect(entry.variants.some((row) => /Should I|Confirm|may I/i.test(row.text))).toBe(true);
  });

  it("bakes German lines for de and de-AT", () => {
    const entry = bakeVariants("rule-1", "block", "default", ["de", "de-AT"]);
    expect(entry.variants.every((row) => /lasse|bleibt|Nicht/i.test(row.text))).toBe(true);
  });

  it("drops speech rows whose rule was deleted", () => {
    const rules: PolicyRule[] = [
      { id: "keep", enabled: true, label: "Keep", when: { domain: "light" }, effect: "confirm" },
    ];
    const pruned = pruneSpeechBank(
      [
        { rule_id: "keep", variants: [{ language: "en", personality: "default", text: "ok" }] },
        { rule_id: "gone", variants: [{ language: "en", personality: "default", text: "bye" }] },
      ],
      rules.map((rule) => rule.id),
    );
    expect(pruned.map((row) => row.rule_id)).toEqual(["keep"]);
  });
});

describe("Lovelace home card (H-25)", () => {
  it("updates fields in place instead of rebuilding the shadow DOM on every hass set", () => {
    const source = fs.readFileSync(
      path.resolve(__dirname, "../../custom_components/klar_nlu/www/klar-home-card.js"),
      "utf8",
    );
    expect(source).toContain("data-field");
    expect(source).toContain("_sync");
    expect(source).not.toMatch(/this\.shadowRoot\.innerHTML\s*=\s*`[\s\S]*\$\{this\._esc/);
  });
});

describe("dead API surface (U-31)", () => {
  it("no longer exports the unused client methods", async () => {
    const { api } = await import("../src/api");
    expect(api).not.toHaveProperty("custom");
    expect(api).not.toHaveProperty("saveCustom");
    expect(api).not.toHaveProperty("previewLang");
    expect(api).not.toHaveProperty("validateProposal");
    expect(api).not.toHaveProperty("conversation");
    expect(api).not.toHaveProperty("deleteBundle");
  });
});
