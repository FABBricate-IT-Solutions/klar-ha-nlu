import { describe, expect, it } from "vitest";
import { assistParseLanguage } from "../src/i18n";

describe("Explain Assist language (U-14)", () => {
  it("uses the pinned Assist language when exactly one is set", () => {
    expect(assistParseLanguage(["de"], "en")).toBe("de");
  });

  it("falls back to the chrome locale when Assist listens to all or several", () => {
    expect(assistParseLanguage([], "en")).toBe("en");
    expect(assistParseLanguage(["de", "fr"], "en")).toBe("en");
  });
});
