import type { Messages } from "../i18n";
import type { ParseResult } from "../types";

const STAGES = ["stageTokens", "stageBind", "stageRank", "stagePolicy", "stageBand"] as const;

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
        {STAGES.map((key, index) => (
          <div className="card" key={key}>
            <h3>{t[key]}</h3>
            <p className={index === 4 || index === 2 ? "mono intent-name" : "muted"}>{cells[index]}</p>
            {result.trace.stages[index] && <p className="caption">{result.trace.stages[index].duration_us} {t.unitsUs}</p>}
          </div>
        ))}
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
