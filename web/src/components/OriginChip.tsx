import type { Messages } from "../i18n";

export type RuleOrigin = "engine" | "seed" | "operator" | "trainer";

export function parseOrigin(raw?: string | null): RuleOrigin | undefined {
  switch (raw) {
    case "engine":
    case "seed":
    case "operator":
    case "trainer":
      return raw;
    default:
      return undefined;
  }
}

function originLabel(t: Messages, origin: RuleOrigin): string {
  switch (origin) {
    case "engine":
      return t.originEngine;
    case "seed":
      return t.originSeed;
    case "operator":
      return t.originOperator;
    case "trainer":
      return t.originTrainer;
    default: {
      const _never: never = origin;
      return _never;
    }
  }
}

export function OriginChip({ t, origin }: { t: Messages; origin: RuleOrigin }) {
  return <span className={`chip origin origin-${origin}`}>{originLabel(t, origin)}</span>;
}
