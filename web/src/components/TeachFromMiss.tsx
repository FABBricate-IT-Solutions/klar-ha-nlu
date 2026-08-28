import type { Messages } from "../i18n";
import type { ConversationTurn } from "../types";

const MISS_BANDS = new Set(["chat", "reject", "clarify"]);
export const TEACH_HEARD_KEY = "klar_teach_heard";
export const TEACH_INTENT_KEY = "klar_teach_intent";

function isDe(t: Messages): boolean {
  return t.replay === "Nochmal";
}

export function rememberAsPhrase(t: Messages): string {
  return isDe(t) ? "Als Phrase merken" : "Remember as phrase";
}

export function isLowConfidence(value: number): boolean {
  if (!Number.isFinite(value)) return false;
  const pct = value > 0 && value <= 1 ? value * 100 : value;
  return pct < 50;
}

export function canTeachFromMiss(turn: ConversationTurn): boolean {
  return MISS_BANDS.has(turn.decision) || isLowConfidence(turn.confidence);
}

export function teachIntentFromNames(names: string[]): string | undefined {
  return names.find((name) => name.startsWith("Hass") || name.startsWith("Klar"));
}

export function openTeach(heard: string, onTeach?: (heard: string) => void, intent?: string): void {
  const phrase = heard.trim();
  if (!phrase) return;
  if (onTeach) {
    onTeach(phrase);
    return;
  }
  sessionStorage.setItem(TEACH_HEARD_KEY, phrase);
  if (intent) sessionStorage.setItem(TEACH_INTENT_KEY, intent);
  window.location.hash = "#/rules";
}

export function TeachFromMiss({
  heard,
  t,
  onReplay,
  onTeach,
  intent,
}: {
  heard: string;
  t: Messages;
  onReplay: (text: string) => void;
  onTeach?: (heard: string) => void;
  intent?: string;
}) {
  const disabled = !heard.trim();
  return (
    <div className="row">
      <button className="secondary" type="button" disabled={disabled} onClick={() => openTeach(heard, onTeach, intent)}>
        {rememberAsPhrase(t)}
      </button>
      <button className="ghost" type="button" disabled={disabled} onClick={() => onReplay(heard)}>{t.inLab}</button>
    </div>
  );
}
