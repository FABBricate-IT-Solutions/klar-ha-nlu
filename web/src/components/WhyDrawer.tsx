import { Drawer } from "./common";
import type { Messages } from "../i18n";
import type { ConversationTurn } from "../types";

export function journalHeard(turn: ConversationTurn): string {
  const consent = typeof turn.text === "string" ? turn.text.trim() : "";
  if (consent) return consent;
  return (turn.tokens ?? []).join(" ").trim();
}

export function canJournalReplay(turn: ConversationTurn): boolean {
  if (typeof turn.text === "string" && turn.text.trim()) return true;
  return (turn.tokens ?? []).length > 0;
}

function isDe(t: Messages): boolean {
  return t.replay === "Nochmal";
}

export function whyThisBand(t: Messages): string {
  return isDe(t) ? "Warum diese Band?" : "Why this band?";
}

function formatConfidence(value: number): string {
  if (!Number.isFinite(value)) return "—";
  const pct = value > 0 && value <= 1 ? value * 100 : value;
  return `${Math.round(pct)}%`;
}

export function WhyDrawer({
  turn,
  t,
  onClose,
}: {
  turn: ConversationTurn;
  t: Messages;
  onClose: () => void;
}) {
  const de = isDe(t);
  const kinds = turn.evidence_kinds ?? [];
  const names = turn.last_names ?? [];
  return (
    <Drawer title={whyThisBand(t)} onClose={onClose} closeLabel={t.close}>
      <label>{t.finalBand}</label>
      <p>
        <span className={`chip ${turn.decision === "execute" ? "intent" : ""}`}>{turn.decision}</span>
      </p>
      <label>{t.confidence}</label>
      <p>{formatConfidence(turn.confidence)}</p>
      <label>{de ? "Evidenz" : "Evidence"}</label>
      <div className="row">
        {kinds.length > 0
          ? kinds.map((kind) => <span className="chip" key={kind}>{kind}</span>)
          : <span className="muted">—</span>}
      </div>
      <label>{de ? "Namen" : "Names"}</label>
      {names.length > 0 ? <p className="mono">{names.join(" · ")}</p> : <p className="muted">—</p>}
      {turn.preferred_area ? (
        <>
          <label>{t.heardIn}</label>
          <p>{turn.preferred_area}</p>
        </>
      ) : null}
      {turn.confirm_prompt ? (
        <>
          <label>{t.effectConfirm}</label>
          <p>{turn.confirm_prompt}</p>
        </>
      ) : null}
    </Drawer>
  );
}
