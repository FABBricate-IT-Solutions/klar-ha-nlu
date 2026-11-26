import { describe, expect, it } from "vitest";
import { decodeHashPart, parseHash } from "../src/routes";

describe("parseHash", () => {
  it("maps a settings deep link to its view", () => {
    expect(parseHash("#/settings/engine")).toEqual({ tab: "settings", settings_view: "engine" });
    expect(parseHash("#/settings/doctor")).toEqual({ tab: "settings", settings_view: "doctor" });
  });

  it("keeps a malformed device id instead of throwing (U-19)", () => {
    expect(decodeHashPart("%E0%A4%A")).toBe("%E0%A4%A");
    expect(parseHash("#/house/devices/%E0%A4%A")).toEqual({
      tab: "house",
      house_view: "entities",
      entity_id: "%E0%A4%A",
    });
  });
});
