import type { ReactNode } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Kpi } from "./common";
import { Guide } from "./Guide";
import { PolicyPath } from "./PolicyPath";
import type { Messages } from "../i18n";
import type { PolicyTrace } from "../types";

export type LotseViewKind =
  | "guide"
  | "architecture"
  | "path"
  | "gaps"
  | "entity"
  | "house"
  | "matchers"
  | "policies"
  | "lexicon"
  | "areas"
  | "engine"
  | "counts"
  | "validate"
  | "write"
  | "languages"
  | "phrases"
  | "turns";

export type LotseViewSpec = {
  kind: LotseViewKind;
  payload: Record<string, unknown>;
};

function asRows(value: unknown): Record<string, unknown>[] {
  return Array.isArray(value) ? value.filter((row): row is Record<string, unknown> => Boolean(row) && typeof row === "object") : [];
}

function asStrings(value: unknown): string[] {
  return Array.isArray(value) ? value.map((row) => String(row)) : [];
}

function textOf(row: Record<string, unknown>, key: string): string {
  const value = row[key];
  return value == null ? "" : String(value);
}

function asTrace(value: unknown): PolicyTrace | null {
  return value && typeof value === "object" ? (value as PolicyTrace) : null;
}

function flagOn(value: unknown): boolean {
  return value === true;
}

function Panel({ title, children }: { title?: string; children: ReactNode }) {
  return (
    <Card size="sm">
      {title ? (
        <CardHeader>
          <CardTitle>{title}</CardTitle>
        </CardHeader>
      ) : null}
      <CardContent className="grid gap-3">{children}</CardContent>
    </Card>
  );
}

function Chips({ items, hot }: { items: string[]; hot?: boolean }) {
  if (items.length === 0) {
    return <p className="muted">—</p>;
  }
  return (
    <div className="flex flex-wrap gap-2">
      {items.map((item) => (
        <span className={hot ? "chip on" : "chip"} key={item}>{item}</span>
      ))}
    </div>
  );
}

function Flag({ on, label }: { on: boolean; label: string }) {
  return <span className={on ? "chip on" : "chip"}>{label}</span>;
}

function Rows({ rows, primary, secondary }: { rows: Record<string, unknown>[]; primary: string; secondary?: string }) {
  if (rows.length === 0) {
    return <p className="muted">—</p>;
  }
  return (
    <ul className="trainer-card-list">
      {rows.slice(0, 16).map((row, index) => (
        <li className="list-row" key={`${textOf(row, primary)}-${index}`}>
          <span>{textOf(row, primary)}</span>
          {secondary && textOf(row, secondary) ? <code className="mono">{textOf(row, secondary)}</code> : null}
        </li>
      ))}
    </ul>
  );
}

function MatcherRows({ rows }: { rows: Record<string, unknown>[] }) {
  if (rows.length === 0) {
    return <p className="muted">—</p>;
  }
  return (
    <ul className="trainer-card-list">
      {rows.slice(0, 16).map((row, index) => (
        <li className="list-row" key={`${textOf(row, "id")}-${index}`}>
          <span className="mono">{textOf(row, "id")}</span>
          <Flag on={flagOn(row.enabled)} label={textOf(row, "precedence") || (flagOn(row.enabled) ? "on" : "off")} />
        </li>
      ))}
    </ul>
  );
}

export function inferLotseView(name: string): LotseViewKind {
  switch (name) {
    case "list_gaps":
      return "gaps";
    case "list_matchers":
      return "matchers";
    case "list_policies":
      return "policies";
    case "list_languages":
      return "languages";
    case "list_lexicon_paths":
    case "get_lexicon":
      return "lexicon";
    case "search_house":
      return "house";
    case "get_entity":
      return "entity";
    case "validate_proposal":
      return "validate";
    case "explain_klar":
      return "architecture";
    case "try_sentence":
      return "path";
    case "list_areas":
      return "areas";
    case "count_house":
      return "counts";
    case "list_engine":
      return "engine";
    case "list_phrases":
      return "phrases";
    case "list_turns":
      return "turns";
    case "apply_lexicon":
    case "apply_match":
    case "apply_house":
    case "apply_aliases":
    case "apply_engine":
    case "apply_ui":
      return "write";
    default:
      return "guide";
  }
}

export function parseLotseViewKind(raw: string): LotseViewKind | null {
  switch (raw) {
    case "guide":
    case "architecture":
    case "path":
    case "gaps":
    case "entity":
    case "house":
    case "matchers":
    case "policies":
    case "lexicon":
    case "areas":
    case "engine":
    case "counts":
    case "validate":
    case "write":
    case "languages":
    case "phrases":
    case "turns":
      return raw;
    default:
      return null;
  }
}

export function LotseView({ spec, t }: { spec: LotseViewSpec; t: Messages }) {
  const { kind, payload } = spec;
  switch (kind) {
    case "guide":
    case "architecture":
      return (
        <Guide
          title={textOf(payload, "title") || t.trainer}
          steps={asRows(payload.steps).map((row, index) => ({
            id: textOf(row, "id") || String(index),
            label: textOf(row, "label"),
            hint: textOf(row, "hint"),
          }))}
        />
      );
    case "path":
      return (
        <Panel title={textOf(payload, "speech") || t.speech}>
          {textOf(payload, "speech") ? <p>{textOf(payload, "speech")}</p> : null}
          <PolicyPath t={t} trace={asTrace(payload.policy_trace)} />
        </Panel>
      );
    case "gaps":
      return <Panel title={t.coverageOpen}><Rows rows={asRows(payload.gaps)} primary="name" secondary="entity_id" /></Panel>;
    case "entity":
      return (
        <Panel title={textOf(payload, "name") || t.entities}>
          <p className="mono">{textOf(payload, "entity_id")}</p>
          {textOf(payload, "area") ? <span className="chip">{textOf(payload, "area")}</span> : <span className="chip">{t.unmapped}</span>}
          <Chips items={asStrings(payload.aliases)} />
        </Panel>
      );
    case "house":
      return (
        <Panel title={t.house}>
          <Rows rows={asRows(payload.entities)} primary="name" secondary="entity_id" />
          <Rows rows={asRows(payload.areas)} primary="name" secondary="area_id" />
        </Panel>
      );
    case "matchers":
      return <Panel title={t.pathMatch}><MatcherRows rows={asRows(payload.matchers)} /></Panel>;
    case "policies":
      return <Panel title={t.pathHouse}><Rows rows={asRows(payload.policies)} primary="label" secondary="id" /></Panel>;
    case "lexicon":
      return (
        <Panel title={textOf(payload, "path") || t.lexiconOverlay}>
          {textOf(payload, "path") ? <p className="mono">{textOf(payload, "path")}</p> : null}
          <Chips items={asStrings(payload.paths)} />
        </Panel>
      );
    case "areas":
      return <Panel title={t.rooms}><Rows rows={asRows(payload.areas)} primary="name" secondary="area_id" /></Panel>;
    case "engine":
      return (
        <Panel title={t.engineReady}>
          <Chips items={asStrings(payload.languages)} hot />
          <div className="flex flex-wrap gap-2">
            {textOf(payload, "personality") ? <span className="chip">{textOf(payload, "personality")}</span> : null}
            {textOf(payload, "mode") ? <span className="chip">{textOf(payload, "mode")}</span> : null}
            <Flag on={flagOn(payload.refine_speech)} label={t.refineSpeech} />
            <Flag on={flagOn(payload.nlu_rag)} label={t.nluRag} />
            <Flag on={flagOn(payload.calendar_llm)} label={t.calendarLlm} />
            <Flag on={flagOn(payload.quiet_ack)} label={t.quietAck} />
            {textOf(payload, "theme") === "light" ? <span className="chip">{t.appearanceLight}</span> : <span className="chip">{t.appearanceDark}</span>}
          </div>
        </Panel>
      );
    case "counts":
      return (
        <div className="grid gap-3 md:grid-cols-3">
          <Kpi value={textOf(payload, "leftover") || "0"} label={t.coverageOpen} hot={Number(textOf(payload, "leftover") || "0") > 0} />
          <Kpi value={textOf(payload, "entities") || "0"} label={t.houseDevices} />
          <Kpi value={textOf(payload, "areas") || "0"} label={t.rooms} />
        </div>
      );
    case "validate":
      return (
        <Panel title={payload.ok === false ? t.trainerFail : t.trainerOk}>
          <Rows rows={asRows(payload.errors)} primary="code" secondary="path" />
          <Rows rows={asRows(payload.warnings)} primary="code" secondary="path" />
        </Panel>
      );
    case "write":
      return <Panel title={payload.ok === false ? t.trainerFail : t.trainerOk}><p className="muted">{payload.ok === false ? t.trainerFail : t.trainerOk}</p></Panel>;
    case "languages":
      return <Panel title={t.languages}><Chips items={asStrings(payload.languages)} hot /></Panel>;
    case "phrases":
      return <Panel title={t.custom}><Rows rows={asRows(payload.phrases)} primary="phrase" secondary="intent" /></Panel>;
    case "turns":
      return <Panel title={t.conversations}><Rows rows={asRows(payload.turns)} primary="label" secondary="decision" /></Panel>;
    default: {
      const _never: never = kind;
      return _never;
    }
  }
}
