import { LotseView, parseLotseViewKind, type LotseViewSpec } from "./LotseView";
import { TrainerMarkdown } from "./TrainerMarkdown";
import type { Messages } from "../i18n";

const VIEW_MARK = "LOTSE_VIEW:";
const CHOICE_MARK = "LOTSE_CHOICES:";

function asChoices(value: unknown): string[] {
  const rows = Array.isArray(value) ? value : value && typeof value === "object" && "choices" in value ? (value as { choices: unknown }).choices : [];
  if (!Array.isArray(rows)) {
    return [];
  }
  return rows
    .map((row) => String(row).trim())
    .filter((row) => row.length > 0 && row.length <= 120)
    .slice(0, 4);
}

function parseChoiceLine(line: string): string[] | null {
  const rest = line.trim().startsWith(CHOICE_MARK) ? line.trim().slice(CHOICE_MARK.length).trim() : "";
  if (!rest) {
    return null;
  }
  try {
    return asChoices(JSON.parse(rest));
  } catch {
    return [];
  }
}

function splitLotseBlocks(text: string): { prose: string; views: LotseViewSpec[]; choices: string[] } {
  const views: LotseViewSpec[] = [];
  const choices: string[] = [];
  const kept: string[] = [];
  for (const line of text.replaceAll("\r\n", "\n").split("\n")) {
    if (line.trim().startsWith(CHOICE_MARK)) {
      choices.push(...(parseChoiceLine(line) ?? []));
      continue;
    }
    const toolAt = line.indexOf("TRAINER_TOOL:");
    if (toolAt >= 0) {
      const before = line.slice(0, toolAt).trim();
      if (before) {
        kept.push(before);
      }
      continue;
    }
    const rest = line.trim().startsWith(VIEW_MARK) ? line.trim().slice(VIEW_MARK.length).trim() : "";
    const space = rest.indexOf("{");
    if (rest && space > 0) {
      const kind = parseLotseViewKind(rest.slice(0, space).trim());
      try {
        const payload = JSON.parse(rest.slice(space)) as unknown;
        if (kind && payload && typeof payload === "object" && !Array.isArray(payload)) {
          views.push({ kind, payload: payload as Record<string, unknown> });
          continue;
        }
      } catch {
        /* keep the line */
      }
    }
    kept.push(line);
  }
  return { prose: kept.join("\n").trim(), views, choices };
}

export function lotseReplyChoices(text: string): string[] {
  return splitLotseBlocks(text).choices;
}

export function visibleLotseText(text: string): string {
  return splitLotseBlocks(text).prose;
}

export function unansweredAssistant(lines: { role: string; content?: string }[]): string {
  for (let index = lines.length - 1; index >= 0; index -= 1) {
    const line = lines[index];
    if (line.role === "user") {
      return "";
    }
    if (line.role === "assistant" && line.content?.trim()) {
      return line.content;
    }
  }
  return "";
}

export function asksLotseQuestion(text: string): boolean {
  const prose = visibleLotseText(text);
  if (!prose) {
    return false;
  }
  if (prose.includes("?") || prose.includes("？")) {
    return true;
  }
  return /möchtest du|möchten sie|soll ich|willst du|kann ich|darf ich|should i|do you want|would you|which |what /i.test(prose);
}

export function lotseFallbackChips(text: string, t: Messages): string[] {
  if (!asksLotseQuestion(text)) {
    return [];
  }
  return [t.trainerYes, t.trainerNo, t.trainerNotNow];
}

export function lotseQuickChips(args: {
  lines: { role: string; content?: string }[];
  busy: boolean;
  consent: boolean;
  consumedFor?: string;
  t: Messages;
}): string[] {
  if (args.consent) {
    return [];
  }
  const text = unansweredAssistant(args.lines);
  if (!text || (args.consumedFor && args.consumedFor === text)) {
    return [];
  }
  const replies = lotseReplyChoices(text);
  return replies.length > 0 ? replies : lotseFallbackChips(text, args.t);
}

export function LotseAnswer({ text, t }: { text: string; t: Messages }) {
  const { prose, views } = splitLotseBlocks(text);
  return (
    <>
      {prose ? <TrainerMarkdown text={prose} /> : null}
      {views.map((spec, index) => (
        <LotseView key={`${spec.kind}-${index}`} spec={spec} t={t} />
      ))}
    </>
  );
}
