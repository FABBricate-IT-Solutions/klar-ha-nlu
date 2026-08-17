import { useEffect, useMemo, useState } from "react";
import { api } from "../api";
import type { Messages } from "../i18n";
import type { ConversationTurn } from "../types";

export function ConversationsPage({ t, onReplay }: { t: Messages; onReplay: (text: string) => void }) {
  const [turns, setTurns] = useState<ConversationTurn[]>([]);
  const [error, setError] = useState("");
  useEffect(() => {
    api.conversations().then(setTurns).catch((err) => setError(String(err)));
  }, []);
  const grouped = useMemo(() => {
    const map = new Map<string, ConversationTurn[]>();
    for (const turn of [...turns].sort((a, b) => b.ts_ms - a.ts_ms)) {
      const list = map.get(turn.conversation_id) || [];
      list.push(turn);
      map.set(turn.conversation_id, list);
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
      {grouped.length === 0 && !error && <p className="muted">{t.noConversations}</p>}
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
                  {turn.last_names.length > 0 && <p className="mono">{turn.last_names.join(" · ")}</p>}
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
