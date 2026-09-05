import type { Messages } from "../i18n";

export type LabLlmPreview = {
  nlu: string;
  chat?: string;
  refined?: string;
  accepted?: boolean;
  error?: string;
  busy: boolean;
};

export function LabSpeechCompare({
  t,
  band,
  briefing,
  preview,
}: {
  t: Messages;
  band: string;
  briefing: boolean;
  preview: LabLlmPreview | null;
}) {
  const nlu = preview?.nlu || "…";
  return (
    <div className="card">
      <h2>{t.speech}</h2>
      <div className="lab-speech-compare">
        <div>
          <span className="chip">{t.speechNlu}</span>
          <p>{nlu}</p>
        </div>
        {preview?.busy ? <p className="muted">{t.trainerStreaming}</p> : null}
        {preview?.error ? <p className="muted">{preview.error}</p> : null}
        {preview?.chat != null ? (
          <div>
            <span className="chip">{t.speechChat}</span>
            <p>{preview.chat || "…"}</p>
          </div>
        ) : null}
        {preview?.refined != null ? (
          <div>
            <span className="chip">{t.speechRefined}</span>
            <p>{preview.refined || "…"}</p>
            {preview.accepted === false ? <p className="caption">{t.refineRejected}</p> : null}
          </div>
        ) : null}
      </div>
      <div className="row">
        <span className={`chip lab-band-chip${band !== "chat" ? " intent" : ""}`}>{band}</span>
        {briefing && <span className="chip">briefing</span>}
      </div>
    </div>
  );
}
