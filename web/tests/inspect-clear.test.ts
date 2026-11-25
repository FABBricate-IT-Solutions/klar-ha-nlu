import { describe, expect, it } from "vitest";

/** Payload the device drawer sends when clearing aliases and room (U-36). */
function tagPayload(aliasDraft: string, area: string) {
  const aliases = aliasDraft.split(",").map((item) => item.trim()).filter(Boolean);
  return { aliases, area };
}

describe("device dialog clear (U-36)", () => {
  it("sends an empty alias list and an empty area string so the engine can clear", () => {
    expect(tagPayload("  ", "")).toEqual({ aliases: [], area: "" });
    expect(tagPayload("decke,  ", "wohnzimmer")).toEqual({ aliases: ["decke"], area: "wohnzimmer" });
  });
});
