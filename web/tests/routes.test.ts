import { describe, expect, it } from "vitest";
import { parseHash } from "../src/routes";

describe("parseHash", () => {
  it("maps a settings deep link to its view", () => {
    expect(parseHash("#/settings/engine")).toEqual({ tab: "settings", settings_view: "engine" });
  });
});
