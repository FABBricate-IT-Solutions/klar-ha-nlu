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

function parseJsonObject(raw: string): Record<string, unknown> | null {
  const attempts = [raw.trim()];
  if (raw.includes("\\\"")) {
    attempts.push(raw.replaceAll("\\\"", "\""));
  }
  for (const candidate of attempts) {
    try {
      const payload = JSON.parse(candidate) as unknown;
      if (payload && typeof payload === "object" && !Array.isArray(payload)) {
        return payload as Record<string, unknown>;
      }
    } catch {
      /* try next */
    }
  }
  return null;
}

function unwrap(text: string): string {
  return text.replace(/^[\s`*_]+|[\s`*_]+$/g, "");
}

function takeMark(line: string, mark: string): { before: string; rest: string } | null {
  const at = line.indexOf(mark);
  if (at < 0) {
    return null;
  }
  return { before: unwrap(line.slice(0, at)), rest: unwrap(line.slice(at + mark.length)) };
}

function isMarkStub(line: string): boolean {
  const trim = unwrap(line);
  return /^(LOTSE_VIEW|LOTSE_CHOICES|TRAINER_TOOL)\b/.test(trim);
}

function isPartialMark(line: string): boolean {
  const trim = unwrap(line);
  if (!trim) {
    return false;
  }
  return ["LOTSE_VIEW", "LOTSE_CHOICES", "TRAINER_TOOL"].some((mark) => mark.startsWith(trim) || trim.startsWith(mark));
}

function splitLotseBlocks(text: string): { prose: string; views: LotseViewSpec[]; choices: string[] } {
  const views: LotseViewSpec[] = [];
  const choices: string[] = [];
  const kept: string[] = [];
  for (const line of text.replaceAll("\r\n", "\n").split("\n")) {
    const choice = takeMark(line, CHOICE_MARK);
    if (choice) {
      if (choice.before) kept.push(choice.before);
      choices.push(...(parseChoiceLine(`${CHOICE_MARK} ${choice.rest}`) ?? []));
      continue;
    }
    const tool = takeMark(line, "TRAINER_TOOL:");
    if (tool) {
      if (tool.before) kept.push(tool.before);
      continue;
    }
    const view = takeMark(line, VIEW_MARK);
    if (view) {
      if (view.before) kept.push(view.before);
      const jsonAt = view.rest.search(/[{[]/);
      const kind = parseLotseViewKind((jsonAt >= 0 ? view.rest.slice(0, jsonAt) : view.rest).trim());
      const payload = jsonAt >= 0 ? parseJsonObject(view.rest.slice(jsonAt)) : {};
      if (kind && payload && Object.keys(payload).length) {
        views.push({ kind, payload });
      }
      continue;
    }
    if (isMarkStub(line)) {
      continue;
    }
    kept.push(line);
  }
  const last = kept.at(-1);
  if (last && isPartialMark(last)) {
    kept.pop();
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
