import { inferLotseView, LotseView, parseLotseViewKind } from "./LotseView";
import type { Messages } from "../i18n";

function asRecord(raw: string | undefined): Record<string, unknown> {
  if (!raw?.trim()) {
    return {};
  }
  try {
    const value = JSON.parse(raw) as unknown;
    return value && typeof value === "object" && !Array.isArray(value) ? (value as Record<string, unknown>) : {};
  } catch {
    return {};
  }
}

export function TrainerToolCard({
  name,
  args,
  result,
  t,
}: {
  name: string;
  args: string;
  result?: string;
  t: Messages;
}) {
  const parsedArgs = asRecord(args);
  const parsed = asRecord(result);
  const kind = parseLotseViewKind(typeof parsed.view === "string" ? parsed.view : "") ?? inferLotseView(name);
  return (
    <article className="trainer-card">
      <p className="trainer-kicker">{name}{parsedArgs.q ? ` · ${String(parsedArgs.q)}` : ""}</p>
      <LotseView spec={{ kind, payload: Object.keys(parsed).length ? parsed : parsedArgs }} t={t} />
    </article>
  );
}
