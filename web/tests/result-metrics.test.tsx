import { screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { en } from "../src/i18n/en";
import { DashboardPage } from "../src/pages/Dashboard";
import type { Dashboard, ResultWindow } from "../src/types";
import { fakeEngine } from "./support/fakeEngine";
import { storedDashboard } from "./support/fixtures";
import { renderUi } from "./support/render";

function weeklyResults(): ResultWindow {
  return {
    total: 42,
    window_ms: 7 * 24 * 60 * 60 * 1000,
    decision_mix: { execute: 20, confirm: 2, clarify: 3, reject: 15, chat: 2, error: 0 },
    top_rejections: [
      { reason: "no_action", count: 8 },
      { reason: "no_target", count: 5 },
      { reason: "out_of_domain", count: 4 },
      { reason: "ambiguous", count: 3 },
      { reason: "unsafe", count: 2 },
      { reason: "unsupported", count: 2 },
      { reason: "empty_input", count: 1 },
      { reason: "invalid_input", count: 1 },
      { reason: "placeholder_a", count: 1 },
      { reason: "placeholder_b", count: 1 },
    ],
    repeats_within_30s: 3,
    matcher_hit_rates: [{ id: "grounded_entities", hits: 10, attempts: 12, rate: 10 / 12 }],
  };
}

function home(data: Dashboard) {
  return (
    <DashboardPage
      data={data}
      t={en}
      locale="en"
      dismissed={[]}
      onReplay={() => undefined}
      onApply={() => undefined}
      onOpenCalibrate={() => undefined}
      canApply={false}
      onTeach={() => undefined}
      lastTurn={undefined}
      parseLanguage="en"
    />
  );
}

describe("result metrics on Home (6.8)", () => {
  it("shows the ten most frequent rejections of the week", async () => {
    fakeEngine();
    const results = weeklyResults();
    renderUi(home({ ...storedDashboard, results }));

    expect(await screen.findByText(en.topRejections)).toBeInTheDocument();
    const list = screen.getByRole("list", { name: en.topRejections });
    const items = within(list).getAllByRole("listitem");
    expect(items).toHaveLength(10);
    expect(items[0]).toHaveTextContent("no_action");
    expect(items[0]).toHaveTextContent("8");
    expect(items[9]).toHaveTextContent("placeholder_b");
    for (const row of results.top_rejections) {
      expect(screen.getByTestId(`top-rejection-${row.reason}`)).toBeInTheDocument();
    }
    expect(screen.getByTestId("repeats-within-30s")).toHaveTextContent("3");
    expect(screen.getByText(en.matcherHitRates)).toBeInTheDocument();
  });
});
