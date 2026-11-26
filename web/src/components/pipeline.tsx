import type { Messages } from "../i18n";
import type { ParseResult } from "../types";

const STAGES = ["stageTokens", "stageBind", "stageRank", "stagePolicy", "stageBand"] as const;

/** Engine stage names that feed each Lab column (U-17). */
const STAGE_NAMES: Record<(typeof STAGES)[number], string[]> = {
  stageTokens: ["normalize", "features"],
  stageBind: ["action_candidates", "target_resolution", "binding"],
  stageRank: ["ranking", "semantic_adapter"],
  stagePolicy: ["safety_decision", "mood_guard", "planning"],
  stageBand: [],
};

function durationUs(result: ParseResult, names: string[]): number | null {
  if (names.length === 0) return null;
  let total = 0;
  let hit = false;
  for (const row of result.trace.stages) {
    if (!names.includes(row.stage)) continue;
    total += row.duration_us;
    hit = true;
  }
  return hit ? total : null;
}

export function Pipeline({ result, t }: { result: ParseResult; t: Messages }) {
  const band = result.decision.type;
  const tokens = result.trace.tokens?.join(" · ") || result.trace.normalized || result.text;
  const bind = result.evidence.map((item) => item.value).slice(0, 4).join(", ") || "—";
  const rank = result.plan?.steps[0]?.intent.name || result.evidence[0]?.source || "—";
  const policy = result.policy_trace?.hit || result.policy_trace?.matched_rule || "compiled";
  const cells = [tokens, bind, rank, policy, band];
  const trigger = result.trace.trigger_word?.trim();
  const consumed = result.trace.consumed_spans ?? [];
  const remaining = result.trace.remaining_tokens ?? [];
  const discardedClauses = result.trace.discarded_clauses ?? [];
  return (
    <div>
      <div className="pipeline">
        {STAGES.map((key, index) => {
          const us = durationUs(result, STAGE_NAMES[key]);
          return (
            <div className="card" key={key}>
              <h3>{t[key]}</h3>
              <p className={index === 4 || index === 2 ? "mono intent-name" : "muted"}>{cells[index]}</p>
              {us !== null ? <p className="caption">{us} {t.unitsUs}</p> : null}
            </div>
          );
        })}
      </div>
      {(trigger || consumed.length > 0 || remaining.length > 0 || discardedClauses.length > 0) && (
        <div className="card" style={{ marginTop: 12 }}>
          <h3>{t.decisionTrace}</h3>
          {trigger ? (
            <p className="mono">
              {t.triggerWord}: {trigger}
            </p>
          ) : null}
          {consumed.length > 0 ? (
            <p className="muted">
              {t.consumedSpans}: {consumed.map((span) => span.text).join(" · ")}
            </p>
          ) : null}
          {remaining.length > 0 ? (
            <p className="muted">
              {t.remainingTokens}: {remaining.join(" · ")}
            </p>
          ) : null}
          {discardedClauses.map((row) => (
            <p className="muted" key={`${row.reason}:${row.text}`}>
              {t.discardedClause}: {row.text} · {row.reason}
            </p>
          ))}
        </div>
      )}
      {result.trace.discarded.length > 0 && (
        <div className="card" style={{ marginTop: 12 }}>
          <h3>{t.discarded}</h3>
          {result.trace.discarded.map((row) => (
            <p className="muted" key={row.candidate_id}>{row.policy} · {row.reason} · {row.score.toFixed(2)}</p>
          ))}
        </div>
      )}
    </div>
  );
}
