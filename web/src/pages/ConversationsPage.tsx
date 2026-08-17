import { useEffect, useMemo, useState } from "react";
import { api } from "../api";
import type { Messages } from "../i18n";
import type { ConversationTurn } from "../types";

function asTurns(rows: unknown): ConversationTurn[] {
  if (!Array.isArray(rows)) {
    return [];
  }
  return rows.filter((row): row is ConversationTurn => !!row && typeof row === "object");
}

export function ConversationsPage({ t, onReplay }: { t: Messages; onReplay: (text: string) => void }) {
  const [turns, setTurns] = useState<ConversationTurn[] | null>(null);
  const [error, setError] = useState("");
  useEffect(() => {
    api.conversations()
      .then((rows) => setTurns(asTurns(rows)))
      .catch((err) => {
        setTurns([]);
        setError(String(err));
      });
  }, []);
  const grouped = useMemo(() => {
    const map = new Map<string, ConversationTurn[]>();
    for (const turn of [...(turns || [])].sort((a, b) => (b.ts_ms || 0) - (a.ts_ms || 0))) {
      const id = turn.conversation_id || "unknown";
      const list = map.get(id) || [];
      list.push(turn);
      map.set(id, list);
    }
    return [...map.entries()];
  }, [turns]);
  return (
    <div className="page">
      <section className="hero">
        <div>
          <h1>{t.conversations}</h1>
          <p className="muted">{t.journalHint}</p>
        </div>
      </section>
      {error && <div className="card danger">{error}</div>}
      {turns === null && !error && <div className="card">{t.loading}</div>}
      {turns && grouped.length === 0 && !error && <div className="card">{t.noConversations}</div>}
      <div className="timeline">
        {grouped.map(([id, items]) => (
          <section className="card" key={id} style={{ marginBottom: 16 }}>
            <h3 className="mono">{id}</h3>
            {items.map((turn) => (
              <div className="timeline-item" key={`${id}-${turn.ts_ms}`}>
                <div className="muted">{new Date(turn.ts_ms).toLocaleString()}</div>
                <div>
                  <span className={`chip ${turn.decision === "execute" ? "intent" : ""}`}>{turn.decision}</span>
                  {turn.text && <p>{turn.text}</p>}
                  <p className="muted">{turn.speech || turn.confirm_prompt || ""}</p>
                  {(turn.last_names ?? []).length > 0 && <p className="mono">{turn.last_names.join(" · ")}</p>}
                </div>
                <button className="ghost" onClick={() => onReplay(turn.text || "")} disabled={!turn.text}>{t.replay}</button>
              </div>
            ))}
          </section>
        ))}
      </div>
    </div>
  );
}
