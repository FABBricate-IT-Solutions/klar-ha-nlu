import { screen, waitFor } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { App } from "../src/App";
import { en } from "../src/i18n/en";
import { RulesPage } from "../src/pages/RulesPage";
import { fakeEngine } from "./support/fakeEngine";
import { storedPolicies, storedSettings } from "./support/fixtures";
import { renderUi } from "./support/render";

describe("U-07: settings draft stays local until Save", () => {
  it("does not flip topbar badges when a toggle is flipped without saving", async () => {
    fakeEngine();
    window.history.replaceState(null, "", "/#/settings/engine");
    const { user } = renderUi(<App />);
    const toggle = await screen.findByRole("switch", { name: en.nluRag });
    expect(toggle).toHaveAttribute("aria-checked", "false");
    expect(screen.getByText(en.chatMode)).toBeInTheDocument();

    await user.click(toggle);

    expect(toggle).toHaveAttribute("aria-checked", "true");
    expect(screen.getByText(en.unsavedChanges)).toBeInTheDocument();
    expect(screen.getByText(en.chatMode)).toBeInTheDocument();
    expect(screen.queryByText(en.ragModeShort)).not.toBeInTheDocument();
  });

  it("saves the draft and then updates the topbar", async () => {
    const engine = fakeEngine();
    window.history.replaceState(null, "", "/#/settings/engine");
    const { user } = renderUi(<App />);
    const toggle = await screen.findByRole("switch", { name: en.nluRag });

    await user.click(toggle);
    await user.click(screen.getByRole("button", { name: en.save }));

    expect(await screen.findByText(en.saveOk)).toBeInTheDocument();
    expect(engine.stored.settings).toEqual({ ...storedSettings, nlu_rag: true });
    await waitFor(() => expect(screen.getByText(en.ragModeShort)).toBeInTheDocument());
  });
});

describe("U-05: trainer-written rules survive the next Save", () => {
  it("refetches policies when Lotse applies a house rule", async () => {
    const engine = fakeEngine();
    const { user } = renderUi(
      <RulesPage t={en} locale="en" personality="default" languages={["de"]} rulesView="policies" />,
    );
    await screen.findByText("No AC at night");

    engine.stored.policies = {
      ...storedPolicies,
      policies: [
        ...storedPolicies.policies,
        { id: "lotse-garage", enabled: true, label: "Garage asks first (lotse)", when: { entity_id: "cover.wohnzimmer_rollo" }, effect: "confirm" },
      ],
    };
    window.dispatchEvent(new CustomEvent("klar-lotse-applied", { detail: { tool: "apply_house" } }));

    expect(await screen.findByText(en.rulesReloaded)).toBeInTheDocument();
    expect(await screen.findByText("Garage asks first (lotse)")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: en.save }));
    expect(await screen.findByText(en.saveOk)).toBeInTheDocument();
    expect(engine.stored.policies.policies.map((rule) => rule.id)).toContain("lotse-garage");
  });
});
