import { screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { en } from "../src/i18n/en";
import { DashboardPage } from "../src/pages/Dashboard";
import type { Dashboard } from "../src/types";
import { fakeEngine } from "./support/fakeEngine";
import { storedDashboard } from "./support/fixtures";
import { renderUi } from "./support/render";

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

describe("the LLM card on Home", () => {
  it("says that the LLM was not called yet", async () => {
    fakeEngine();
    renderUi(home(storedDashboard));

    expect(await screen.findByText(en.llmCalls)).toBeInTheDocument();
    expect(screen.getByText(en.llmNoCalls)).toBeInTheDocument();
  });

  it("shows errors, acceptance rate and token use once the LLM was called", async () => {
    fakeEngine();
    renderUi(home({
      ...storedDashboard,
      llm: {
        calls: 4,
        errors: 1,
        accepted: 3,
        rejected: 1,
        by_kind: { refine: 4 },
        by_day: [{ day: "d1", refine: 4, assist: 0, chat: 0 }],
        p50_ms: 310,
        p90_ms: 900,
        tokens: { prompt: 1200, completion: 300, total: 1500, calls_with_usage: 4 },
      },
    }));

    expect(await screen.findByText(`${en.llmErrors}: 1`)).toBeInTheDocument();
    expect(screen.getByText(`${en.llmAcceptRate}: 75${en.unitsPercent}`)).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: en.llmTokens })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: en.llmLatency })).toBeInTheDocument();
    expect(screen.queryByText(en.llmNoCalls)).not.toBeInTheDocument();
  });
});
