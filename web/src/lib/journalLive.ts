import type { ConversationTurn } from "../types";

export type JournalLiveEvent = {
  type: "snapshot" | "turn";
  turn: ConversationTurn;
};

export function mergeJournalTurn(rows: ConversationTurn[], turn: ConversationTurn): ConversationTurn[] {
  const key = `${turn.conversation_id}:${turn.ts_ms}`;
  const without = rows.filter((row) => `${row.conversation_id}:${row.ts_ms}` !== key);
  return [...without, turn].sort((a, b) => (a.ts_ms || 0) - (b.ts_ms || 0));
}

export function turnMatchesQuery(turn: ConversationTurn, query: string, decision: string): boolean {
  if (decision && turn.decision !== decision) return false;
  const needle = query.trim().toLowerCase();
  if (!needle) return true;
  const hay = [
    turn.text || "",
    (turn.tokens || []).join(" "),
    turn.speech || "",
    turn.decision,
    ...(turn.last_names || []),
    ...(turn.evidence_kinds || []),
    turn.preferred_area || "",
  ]
    .join(" ")
    .toLowerCase();
  return hay.includes(needle);
}

export function parseJournalLivePayload(payload: string): JournalLiveEvent | null {
  try {
    const parsed = JSON.parse(payload) as JournalLiveEvent;
    if (parsed && (parsed.type === "snapshot" || parsed.type === "turn") && parsed.turn) {
      return parsed;
    }
  } catch {
    /* skip */
  }
  return null;
}
