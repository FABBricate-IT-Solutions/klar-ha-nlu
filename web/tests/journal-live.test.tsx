import { screen, waitFor } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { en } from "../src/i18n/en";
import { mergeJournalTurn, parseJournalLivePayload, turnMatchesQuery } from "../src/lib/journalLive";
import { ConversationsPage } from "../src/pages/ConversationsPage";
import type { ConversationTurn } from "../src/types";
import { renderUi } from "./support/render";

function turn(partial: Partial<ConversationTurn> & Pick<ConversationTurn, "conversation_id" | "ts_ms" | "decision">): ConversationTurn {
  return {
    text: null,
    tokens: [],
    speech: "",
    confidence: 0.5,
    briefing: false,
    evidence_kinds: [],
    last_names: [],
    ...partial,
  };
}

describe("journal live helpers", () => {
  it("merges a turn by conversation and timestamp", () => {
    const first = turn({ conversation_id: "c1", ts_ms: 1, decision: "chat", text: "a" });
    const second = turn({ conversation_id: "c1", ts_ms: 2, decision: "execute", text: "b" });
    const updated = turn({ conversation_id: "c1", ts_ms: 1, decision: "chat", text: "a!", speech: "ok" });
    expect(mergeJournalTurn([first], second)).toEqual([first, second]);
    expect(mergeJournalTurn([first, second], updated)[0].speech).toBe("ok");
  });

  it("filters by decision and free-text query", () => {
    const row = turn({
      conversation_id: "c1",
      ts_ms: 1,
      decision: "reject",
      text: "open the gate",
      last_names: ["HassOpenCover"],
    });
    expect(turnMatchesQuery(row, "gate", "")).toBe(true);
    expect(turnMatchesQuery(row, "gate", "execute")).toBe(false);
    expect(turnMatchesQuery(row, "HassOpen", "reject")).toBe(true);
  });

  it("parses live SSE payloads", () => {
    const row = turn({ conversation_id: "c9", ts_ms: 9, decision: "chat", text: "hi" });
    expect(parseJournalLivePayload(JSON.stringify({ type: "turn", turn: row }))?.turn.text).toBe("hi");
    expect(parseJournalLivePayload("{")).toBeNull();
  });
});

describe("the conversation journal", () => {
  it("shows a new utterance without reloading the page (6.7)", async () => {
    const { rerender } = renderUi(
      <ConversationsPage t={en} locale="en" turns={[]} live={false} onReplay={() => undefined} />,
    );

    expect(screen.getByText(en.noConversations)).toBeInTheDocument();

    const spoken = turn({
      conversation_id: "live-1",
      ts_ms: Date.now(),
      decision: "execute",
      text: "turn on the kitchen light",
      speech: "Okay.",
    });
    // App feeds live SSE turns into props — no navigation / reload.
    rerender(
      <ConversationsPage t={en} locale="en" turns={[spoken]} live onReplay={() => undefined} />,
    );

    await waitFor(() => expect(screen.getByText("turn on the kitchen light")).toBeInTheDocument());
    expect(screen.queryByText(en.noConversations)).not.toBeInTheDocument();
    expect(screen.getByText(en.journalLive)).toBeInTheDocument();
  });
});
