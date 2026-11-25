import { describe, expect, it } from "vitest";

/** Mirror of App applyCandidates filter (U-12). */
function applyCandidates(
  assignment: Array<{
    entity_id: string;
    area: string | null;
    confidence: string;
    suggested_area?: { area_id: string; score: number } | null;
  }>,
  dismissed: string[],
) {
  return assignment.filter((row) => {
    const suggestion = row.suggested_area;
    if (!suggestion || suggestion.score < 3) return false;
    if (row.area === suggestion.area_id) return false;
    if (row.confidence === "high") return false;
    if (dismissed.includes(row.entity_id)) return false;
    return true;
  });
}

describe("apply suggestion candidates (U-12)", () => {
  it("skips devices that already sit in the suggested room", () => {
    const rows = applyCandidates(
      [
        { entity_id: "light.a", area: "kitchen", confidence: "medium", suggested_area: { area_id: "kitchen", score: 5 } },
        { entity_id: "light.b", area: null, confidence: "medium", suggested_area: { area_id: "kitchen", score: 5 } },
        { entity_id: "light.c", area: "hall", confidence: "high", suggested_area: { area_id: "kitchen", score: 5 } },
      ],
      [],
    );
    expect(rows.map((row) => row.entity_id)).toEqual(["light.b"]);
  });
});
