import { useState } from "react";
import { OriginChip } from "./OriginChip";
import { SearchSelect, withCurrent } from "./SearchSelect";
import { policiesEmpty, SetupHint } from "./SetupHint";
import type { Messages } from "../i18n";
import type { PolicyEffect, PolicyRule, SpeechBank } from "../types";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";

const EFFECTS: PolicyEffect[] = ["confirm", "block", "allow", "prefer_entity", "prefer_area", "reply", "script", "template", "llm"];
const ACTION_EFFECTS: PolicyEffect[] = ["reply", "script", "template", "llm"];

function effectLabel(t: Messages, effect: PolicyEffect): string {
  switch (effect) {
    case "confirm":
      return t.effectConfirm;
    case "block":
      return t.effectBlock;
    case "allow":
      return t.effectAllow;
    case "prefer_entity":
      return t.effectPreferEntity;
    case "prefer_area":
      return t.effectPreferArea;
    case "reply":
      return t.effectReply;
    case "script":
      return t.effectScript;
    case "template":
      return t.effectTemplate;
    case "llm":
      return t.effectLlm;
    default: {
      const _never: never = effect;
      return _never;
    }
  }
}

function payloadHint(t: Messages, effect: PolicyEffect): string {
  switch (effect) {
    case "reply":
      return t.payloadReply;
    case "script":
      return t.payloadScript;
    case "template":
      return t.payloadTemplate;
    case "llm":
      return t.payloadLlm;
    case "confirm":
    case "block":
    case "allow":
    case "prefer_entity":
    case "prefer_area":
      return "";
    default: {
      const _never: never = effect;
      return _never;
    }
  }
}

type Option = { value: string; label: string };

export function HouseLane({
  t,
  rules,
  seedIds,
  selected,
  bank,
  intentOptions,
  entityOptions,
  rooms,
  domains,
  floors,
  onSelect,
  onMove,
  onRemove,
  onUpdate,
  onUpdateWhen,
  onUpdateWhenEntity,
  onBake,
}: {
  t: Messages;
  rules: PolicyRule[];
  seedIds: Set<string>;
  selected: number;
  bank: SpeechBank;
  intentOptions: Option[];
  entityOptions: Option[];
  rooms: Option[];
  domains: Option[];
  floors: Option[];
  onSelect: (index: number) => void;
  onMove: (from: number, to: number) => void;
  onRemove: (id: string) => void;
  onUpdate: (patch: Partial<PolicyRule>) => void;
  onUpdateWhen: (key: keyof PolicyRule["when"], value: string) => void;
  onUpdateWhenEntity: (entityId: string) => void;
  onBake: () => void;
}) {
  const houseRules = rules
    .map((rule, index) => ({ rule, index }))
    .filter(({ rule }) => !seedIds.has(rule.id));
  const current = rules[selected];
  const editingHouse = Boolean(current && !seedIds.has(current.id));
  const [pendingDelete, setPendingDelete] = useState<string | null>(null);

  return (
    <>
      <div className="policy-lane-head">
        <h2>{t.laneHouse}</h2>
        <OriginChip t={t} origin="operator" />
      </div>
      <div className="lane-body">
      {houseRules.length === 0 && (
        <div>
          <p className="muted">{policiesEmpty(t)}</p>
          <SetupHint t={t} />
        </div>
      )}
      {houseRules.map(({ rule, index }, order) => (
        <div
          className={`lane-row house-row${index === selected ? " active" : ""}`}
          key={rule.id}
          draggable
          onDragStart={(ev) => ev.dataTransfer.setData("text/plain", String(index))}
          onDragOver={(ev) => ev.preventDefault()}
          onDrop={(ev) => {
            ev.preventDefault();
            onMove(Number(ev.dataTransfer.getData("text/plain")), index);
          }}
          onClick={(ev) => {
            ev.stopPropagation();
            onSelect(index);
          }}
        >
          <span className="muted">{order + 1}</span>
          <strong>{rule.label || rule.id}</strong>
          <span className="chip intent">{rule.effect}</span>
          <OriginChip t={t} origin="operator" />
          <div className="row">
            <button
              className="ghost"
              type="button"
              aria-label={t.moveRuleUp}
              disabled={order === 0}
              onClick={(ev) => {
                ev.stopPropagation();
                onMove(index, houseRules[order - 1]!.index);
              }}
            >
              ↑
            </button>
            <button
              className="ghost"
              type="button"
              aria-label={t.moveRuleDown}
              disabled={order === houseRules.length - 1}
              onClick={(ev) => {
                ev.stopPropagation();
                onMove(index, houseRules[order + 1]!.index);
              }}
            >
              ↓
            </button>
            <button
              className="ghost danger"
              type="button"
              onClick={(ev) => {
                ev.stopPropagation();
                setPendingDelete(rule.id);
              }}
            >
              {t.dismiss}
            </button>
          </div>
        </div>
      ))}
      {editingHouse && current ? (
        <div className="house-editor" onClick={(ev) => ev.stopPropagation()}>
          <label htmlFor="klar-house-label">{t.custom}</label>
          <input id="klar-house-label" value={current.label} onChange={(ev) => onUpdate({ label: ev.target.value })} />
          <label className="row">
            <input type="checkbox" checked={current.enabled} onChange={(ev) => onUpdate({ enabled: ev.target.checked })} />
            {current.enabled ? t.matchEnabled : t.matchDisabled}
          </label>
          <label>{t.when}</label>
          <label htmlFor="klar-house-phrase">{t.whenPhrase}</label>
          <input id="klar-house-phrase" placeholder={t.whenPhrase} value={current.when.phrase || ""} onChange={(ev) => onUpdateWhen("phrase", ev.target.value)} />
          <label htmlFor="klar-house-intent">{t.whenIntent}</label>
          <SearchSelect
            id="klar-house-intent"
            value={current.when.intent || ""}
            options={withCurrent(intentOptions, current.when.intent || "")}
            onChange={(value) => onUpdateWhen("intent", value)}
            placeholder={t.whenIntent}
          />
          <label htmlFor="klar-house-domain">{t.whenDomain}</label>
          <SearchSelect
            id="klar-house-domain"
            value={current.when.domain || ""}
            options={withCurrent(domains, current.when.domain || "")}
            onChange={(value) => onUpdateWhen("domain", value)}
            placeholder={t.whenDomain}
          />
          <label htmlFor="klar-house-area">{t.whenArea}</label>
          <SearchSelect
            id="klar-house-area"
            value={current.when.area || ""}
            options={withCurrent(rooms, current.when.area || "")}
            onChange={(value) => onUpdateWhen("area", value)}
            placeholder={t.whenArea}
          />
          <label htmlFor="klar-house-entity">{t.whenEntity}</label>
          <SearchSelect
            id="klar-house-entity"
            value={current.when.entity_id || ""}
            options={withCurrent(entityOptions, current.when.entity_id || "")}
            onChange={onUpdateWhenEntity}
            placeholder={t.whenEntity}
          />
          <label htmlFor="klar-house-floor">{t.whenFloor}</label>
          <SearchSelect
            id="klar-house-floor"
            value={current.when.floor || ""}
            options={withCurrent(floors, current.when.floor || "")}
            onChange={(value) => onUpdateWhen("floor", value)}
            placeholder={t.whenFloor}
          />
          <label htmlFor="klar-house-name">{t.whenName}</label>
          <input id="klar-house-name" placeholder={t.whenName} value={current.when.name || ""} onChange={(ev) => onUpdateWhen("name", ev.target.value)} />
          <label htmlFor="klar-house-effect">{t.then}</label>
          <select id="klar-house-effect" value={current.effect} onChange={(ev) => onUpdate({ effect: ev.target.value as PolicyEffect })}>
            {EFFECTS.map((effect) => <option key={effect} value={effect}>{effectLabel(t, effect)}</option>)}
          </select>
          {current.effect === "prefer_entity" && (
            <>
              <label htmlFor="klar-house-prefer">{t.whenPrefer}</label>
              <SearchSelect
                id="klar-house-prefer"
                value={current.prefer || ""}
                options={withCurrent(entityOptions, current.prefer || "")}
                onChange={(value) => onUpdate({ prefer: value || undefined })}
                placeholder={t.whenPrefer}
                allowEmpty={false}
              />
            </>
          )}
          {current.effect === "prefer_area" && (
            <>
              <label htmlFor="klar-house-prefer-area">{t.whenPrefer}</label>
              <SearchSelect
                id="klar-house-prefer-area"
                value={current.prefer || ""}
                options={withCurrent(rooms, current.prefer || "")}
                onChange={(value) => onUpdate({ prefer: value || undefined })}
                placeholder={t.whenPrefer}
                allowEmpty={false}
              />
            </>
          )}
          {ACTION_EFFECTS.includes(current.effect) && (
            <>
              <label htmlFor="klar-house-payload">{payloadHint(t, current.effect)}</label>
              <textarea id="klar-house-payload" placeholder={payloadHint(t, current.effect)} value={current.payload || ""} onChange={(ev) => onUpdate({ payload: ev.target.value })} />
            </>
          )}
          <div className="row" style={{ marginTop: 12 }}>
            <button className="secondary" type="button" onClick={onBake}>{t.bakeSpeech}</button>
          </div>
          {bank.entries.find((item) => item.rule_id === current.id)?.variants.map((variant, index) => (
            <p className="muted" key={`${variant.language}-${index}`}>{variant.language}/{variant.personality}: {variant.text}</p>
          ))}
        </div>
      ) : houseRules.length === 0 ? null : <p className="muted">{t.noPolicies}</p>}
      </div>
      <AlertDialog open={pendingDelete !== null} onOpenChange={(open) => { if (!open) setPendingDelete(null); }}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t.confirmDeleteRule}</AlertDialogTitle>
            <AlertDialogDescription>{t.confirmDeleteRuleHint}</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t.cancel}</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                if (pendingDelete) onRemove(pendingDelete);
                setPendingDelete(null);
              }}
            >
              {t.dismiss}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
