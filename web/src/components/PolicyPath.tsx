import { OriginChip, parseOrigin } from "./OriginChip";
import type { Messages } from "../i18n";
import type { PolicyTrace } from "../types";
import { cn } from "cn";

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

function nodeOrigin(trace: PolicyTrace | undefined, kind: PathKind) {
  switch (kind) {
    case "match":
      return parseOrigin(trace?.match?.origin);
    case "seed":
      return parseOrigin(trace?.seed?.origin);
    case "house":
      return parseOrigin(trace?.house?.origin);
    case "band":
      return undefined;
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
    <div className="flex flex-wrap items-stretch gap-2" aria-label={t.pathMatch}>
      {kinds.map((kind, index) => {
        const id = nodeId(policy, kind, dash);
        const lane = laneFor(kind);
        const why = nodeWhy(t, policy, kind);
        const origin = nodeOrigin(policy, kind);
        const hit = id !== dash;
        const selectedId = hit ? id : undefined;
        return (
          <span key={kind} className="inline-flex items-center gap-2">
            {index > 0 ? <span className="text-foreground" aria-hidden="true">→</span> : null}
            <button
              type="button"
              className={cn(
                "flex min-h-11 min-w-[132px] flex-col items-start gap-0.5 rounded-lg border bg-card px-3 py-2 text-start transition-colors",
                hit ? "border-primary ring-1 ring-primary/35" : "border-border",
              )}
              onClick={() => lane && onSelect?.(lane, selectedId)}
              disabled={!lane || !onSelect}
            >
              <span className="flex w-full items-center justify-between gap-2">
                <span className="text-xs text-foreground">{kindLabel(t, kind)}</span>
                {origin && hit ? <OriginChip t={t} origin={origin} /> : null}
              </span>
              <strong className="font-mono text-xs">{id}</strong>
              {why ? <span className="text-[11px] text-foreground">{why}</span> : null}
            </button>
          </span>
        );
      })}
      {policy?.discarded && policy.discarded.length > 0 && (
        <div className="basis-full mt-2">
          <h3>{t.discarded}</h3>
          {policy.discarded.map((row) => (
            <p className="text-sm text-foreground" key={`${row.id}-${row.reason}`}>
              {row.id} · {row.reason} · {row.score.toFixed(2)}
            </p>
          ))}
        </div>
      )}
    </div>
  );
}
