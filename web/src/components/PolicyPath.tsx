import type { Messages } from "../i18n";
import type { PolicyTrace } from "../types";

export type PolicyLane = "match" | "language" | "house";
type PathKind = "match" | "seed" | "house" | "band";

function laneFor(kind: PathKind): PolicyLane | undefined {
  switch (kind) {
    case "match":
      return "match";
    case "seed":
      return "language";
    case "house":
      return "house";
    case "band":
      return undefined;
    default: {
      const _never: never = kind;
      return _never;
    }
  }
}

function kindLabel(t: Messages, kind: PathKind): string {
  switch (kind) {
    case "match":
      return t.pathMatch;
    case "seed":
      return t.pathSeed;
    case "house":
      return t.pathHouse;
    case "band":
      return t.pathBand;
    default: {
      const _never: never = kind;
      return _never;
    }
  }
}

function nodeId(trace: PolicyTrace | undefined, kind: PathKind, dash: string): string {
  switch (kind) {
    case "match":
      return trace?.match?.id || dash;
    case "seed":
      return trace?.seed?.id || dash;
    case "house":
      return trace?.house?.id || trace?.matched_rule || dash;
    case "band":
      return trace?.band || dash;
    default: {
      const _never: never = kind;
      return _never;
    }
  }
}

function nodeWhy(t: Messages, trace: PolicyTrace | undefined, kind: PathKind): string {
  switch (kind) {
    case "match":
      return trace?.match ? `${t.confidence} ${trace.match.score.toFixed(2)}` : "";
    case "seed":
      return trace?.seed?.hit || "";
    case "house":
      return trace?.house?.hit || trace?.hit || "";
    case "band":
      return !trace?.seed && !trace?.house && trace?.compiled_risky ? t.compiledFloor : "";
    default: {
      const _never: never = kind;
      return _never;
    }
  }
}

export function PolicyPath({
  t,
  trace,
  onSelect,
}: {
  t: Messages;
  trace?: PolicyTrace | null;
  onSelect?: (lane: PolicyLane, id?: string) => void;
}) {
  const policy = trace ?? undefined;
  const dash = t.pathUnchecked;
  const kinds: PathKind[] = ["match", "seed", "house", "band"];
  return (
    <div className="policy-path" aria-label={t.processPath}>
      {kinds.map((kind, index) => {
        const id = nodeId(policy, kind, dash);
        const lane = laneFor(kind);
        const why = nodeWhy(t, policy, kind);
        const selectedId = id === dash ? undefined : id;
        return (
          <span key={kind} className="policy-path-step">
            {index > 0 ? <span className="muted"> → </span> : null}
            <button
              type="button"
              className={`policy-path-node${id === dash ? "" : " hit"}`}
              onClick={() => lane && onSelect?.(lane, selectedId)}
              disabled={!lane}
            >
              <span className="muted">{kindLabel(t, kind)}</span>
              <strong className="mono">{id}</strong>
              {why ? <span className="caption">{why}</span> : null}
            </button>
          </span>
        );
      })}
      {policy?.discarded && policy.discarded.length > 0 && (
        <div className="policy-path-discarded">
          <h3>{t.discarded}</h3>
          {policy.discarded.map((row) => (
            <p className="muted" key={`${row.id}-${row.reason}`}>
              {row.id} · {row.reason} · {row.score.toFixed(2)}
            </p>
          ))}
        </div>
      )}
    </div>
  );
}
