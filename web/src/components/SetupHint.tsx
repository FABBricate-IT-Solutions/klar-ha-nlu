import type { Messages } from "../i18n";

export function sentencesEmpty(t: Messages): string {
  return t.sentencesEmpty;
}

export function policiesEmpty(t: Messages): string {
  return t.policiesEmpty;
}

export function setupAgainWhere(t: Messages): string {
  return t.setupAgainWhere;
}

export function SetupHint({ t }: { t: Messages }) {
  return <p className="caption">{setupAgainWhere(t)}</p>;
}
