import { useState } from "react";
import { api, type LangOverlay } from "../api";
import type { Messages } from "../i18n";
import type { MatchControl, PolicyRule, TrainerProposal, TrainerValidateOut } from "../types";

function layerOf(proposal: TrainerProposal): "match" | "language" | "house" | "all" {
  const layer = proposal.layer || "all";
  switch (layer) {
    case "match":
    case "language":
    case "house":
    case "all":
      return layer;
    default:
      return "all";
  }
}

export function TrainerDrawer({
  t,
  language,
  overlay,
  onApplyHouse,
  onApplyMatch,
  onStatus,
}: {
  t: Messages;
  language?: string;
  overlay: LangOverlay | null;
  onApplyHouse: (next: PolicyRule[]) => Promise<void>;
  onApplyMatch: (next: MatchControl[]) => Promise<void>;
  onStatus: (status: string) => void;
}) {
  const [raw, setRaw] = useState("");
  const [result, setResult] = useState<TrainerValidateOut | null>(null);
  const [contextText, setContextText] = useState("");

  const loadContext = async () => {
    const body = await api.trainerContext("all", language);
    setContextText(JSON.stringify(body, null, 2));
    onStatus(t.trainerContext);
  };

  const parsed = (): TrainerProposal | null => {
    if (!raw.trim()) return null;
    try {
      return JSON.parse(raw) as TrainerProposal;
    } catch {
      return null;
    }
  };

  const runValidate = async () => {
    const proposal = parsed();
    if (!proposal) {
      onStatus(t.trainerFail);
      return;
    }
    if (!proposal.language && language) proposal.language = language;
    const out = await api.validateProposal(proposal);
    setResult(out);
    onStatus(out.ok ? t.trainerOk : t.trainerFail);
  };

  const apply = async () => {
    if (!result?.ok) return;
    const proposal = parsed();
    if (!proposal) return;
    const layer = layerOf(proposal);
    if ((layer === "house" || layer === "all") && proposal.policies) {
      await onApplyHouse(proposal.policies);
    }
    if ((layer === "match" || layer === "all") && proposal.match_controls) {
      await onApplyMatch(proposal.match_controls);
    }
    if ((layer === "language" || layer === "all") && proposal.language_overlay) {
      await api.saveLangOverlay({ custom: overlay?.custom || [], language: proposal.language_overlay, label: "trainer" });
    }
    onStatus(t.trainerApply);
  };

  return (
    <div className="card" style={{ marginTop: 16 }}>
      <h2>{t.trainer}</h2>
      <p className="muted">{t.trainerHint}</p>
      <div className="row">
        <button className="secondary" onClick={() => void loadContext()}>{t.trainerContext}</button>
        <button className="primary" onClick={() => void runValidate()}>{t.trainerValidate}</button>
        <button className="secondary" disabled={!result?.ok} onClick={() => void apply()}>{t.trainerApply}</button>
      </div>
      {contextText && <pre className="trainer-json">{contextText}</pre>}
      <label>{t.trainerProposal}</label>
      <textarea className="trainer-json" value={raw} onChange={(ev) => setRaw(ev.target.value)} rows={10} />
      {result && (
        <div style={{ marginTop: 12 }}>
          <p className="muted">{result.ok ? t.trainerOk : t.trainerFail}</p>
          {result.errors.map((row) => (
            <p className="muted" key={`${row.path}-${row.message}`}>{row.path}: {row.message}</p>
          ))}
          {result.warnings.map((row) => (
            <p className="muted" key={`w-${row.path}-${row.message}`}>{row.path}: {row.message}</p>
          ))}
          {result.dry_run.map((row) => (
            <p className="lexicon-delta" key={row.text}>
              {row.text} → {row.decision}
              {row.seed ? ` · seed ${row.seed}` : ""}
              {row.house ? ` · house ${row.house}` : ""}
            </p>
          ))}
        </div>
      )}
    </div>
  );
}
