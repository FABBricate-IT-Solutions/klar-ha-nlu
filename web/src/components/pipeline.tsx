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
