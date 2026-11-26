import { screen, waitFor } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { App } from "../src/App";
import { en } from "../src/i18n/en";
import { fakeEngine } from "./support/fakeEngine";
import { renderUi } from "./support/render";

describe("6.3: update dialog lists affected sentences", () => {
  it("shows changed regression sentences after an engine update", async () => {
    const engine = fakeEngine();
    engine.on("GET /api/v2/wrong-cases/update-dialog", {
      body: {
        engine_version: "2026.11.99",
        baseline_version: "2026.11.1",
        is_update: true,
        total: 2,
        changed: 1,
        cases: [{ id: "c1", text: "Küche Licht an", before: "reject", after: "execute|HassTurnOn[entity_id=light.e01]" }],
      },
    });
    window.history.replaceState(null, "", "/#/");
    const { user } = renderUi(<App />);

    expect(await screen.findByText(en.regressionUpdateTitle)).toBeInTheDocument();
    expect(screen.getByText(en.regressionUpdateCount.replace("{n}", "1"))).toBeInTheDocument();
    expect(screen.getByText("Küche Licht an")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: en.regressionUpdateAck }));
    await waitFor(() => expect(engine.to("POST /api/v2/wrong-cases/update-dialog").length).toBe(1));
    await waitFor(() => expect(screen.queryByText(en.regressionUpdateTitle)).not.toBeInTheDocument());
  });
});
