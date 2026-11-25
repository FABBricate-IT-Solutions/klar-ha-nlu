import fs from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { wizardMessages } from "../src/i18n/wizard";

/** Relative luminance for sRGB hex (#rrggbb). */
function luminance(hex: string): number {
  const raw = hex.replace("#", "");
  const channels = [0, 1, 2].map((i) => {
    const value = Number.parseInt(raw.slice(i * 2, i * 2 + 2), 16) / 255;
    return value <= 0.03928 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * channels[0]! + 0.7152 * channels[1]! + 0.0722 * channels[2]!;
}

function contrast(a: string, b: string): number {
  const left = luminance(a);
  const right = luminance(b);
  const lighter = Math.max(left, right);
  const darker = Math.min(left, right);
  return (lighter + 0.05) / (darker + 0.05);
}

describe("wizard locale key sets (U-16 acceptance)", () => {
  it("keeps the same key set for every wizard JSON pack", () => {
    const dir = path.resolve(__dirname, "../src/i18n/wizard");
    const files = fs.readdirSync(dir).filter((name) => name.endsWith(".json"));
    expect(files.length).toBeGreaterThan(60);
    const base = new Set(Object.keys(JSON.parse(fs.readFileSync(path.join(dir, files[0]!), "utf8")) as object));
    for (const file of files) {
      const keys = new Set(Object.keys(JSON.parse(fs.readFileSync(path.join(dir, file), "utf8")) as object));
      expect([...keys].sort(), file).toEqual([...base].sort());
    }
  });

  it("merges overlays onto the oracle without dropping required titles", () => {
    const fr = wizardMessages("fr");
    expect(fr.whatTitle.length).toBeGreaterThan(0);
    expect(fr.pathTitle.length).toBeGreaterThan(0);
  });
});

describe("theme contrast (U-15)", () => {
  it("keeps body text above 4.5:1 on both themes", () => {
    expect(contrast("#f4efe6", "#1c1c1c")).toBeGreaterThanOrEqual(4.5);
    expect(contrast("#1c1c1c", "#f4efe6")).toBeGreaterThanOrEqual(4.5);
    expect(contrast("#b7b0a4", "#1c1c1c")).toBeGreaterThanOrEqual(4.5);
    expect(contrast("#6b655c", "#f4efe6")).toBeGreaterThanOrEqual(4.5);
  });
});
