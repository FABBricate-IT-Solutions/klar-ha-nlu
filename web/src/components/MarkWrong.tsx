import { useEffect, useMemo, useState } from "react";
import { api } from "../api";
import { SearchSelect, useHouseCatalog, withCurrent } from "./SearchSelect";
import { journalHeard } from "./WhyDrawer";
import type { Messages } from "../i18n";
import type { ConversationTurn } from "../types";

export function MarkWrong({
  turn,
  t,
}: {
  turn: ConversationTurn;
  t: Messages;
}) {
  const [open, setOpen] = useState(false);
  const [intent, setIntent] = useState("HassTurnOn");
  const [target, setTarget] = useState("");
  const [intents, setIntents] = useState<string[]>([]);
  const [status, setStatus] = useState("");
  const [busy, setBusy] = useState(false);
  const { entities, rooms } = useHouseCatalog();
  const heard = journalHeard(turn);

  useEffect(() => {
    api.intents().then((rows) => setIntents(Array.isArray(rows) ? rows : [])).catch(() => undefined);
  }, []);

  const intentOptions = useMemo(
    () => (intents.length ? intents : ["HassTurnOn", "HassTurnOff"]).map((name) => ({ value: name, label: name })),
    [intents],
  );
  const targetOptions = useMemo(() => {
    const deviceRows = entities.map((entity) => ({
      value: entity.entity_id,
      label: entity.name || entity.entity_id,
      hint: entity.entity_id,
    }));
    const roomRows = rooms.map((room) => ({
      value: `area:${room.value}`,
      label: room.label,
      hint: room.value,
    }));
    return [...deviceRows, ...roomRows];
  }, [entities, rooms]);

  const save = async () => {
    if (!heard.trim() || !target) return;
    setBusy(true);
    setStatus("");
    try {
      const slots: Record<string, string> = {};
      if (target.startsWith("area:")) {
        slots.area = target.slice("area:".length);
      } else {
        slots.entity_id = target;
        const domain = target.split(".")[0];
        if (domain) slots.domain = domain;
      }
      const out = await api.markWrong({
        conversation_id: turn.conversation_id,
        ts_ms: turn.ts_ms,
        text: heard,
        intent,
        slots,
      });
      setStatus(t.markWrongSaved);
      const blob = new Blob([out.case_yaml], { type: "application/yaml;charset=utf-8" });
      const url = URL.createObjectURL(blob);
      const link = document.createElement("a");
      link.href = url;
      link.download = `klar-wrong-${out.id}.yaml`;
      link.click();
      URL.revokeObjectURL(url);
      setOpen(false);
    } catch (err) {
      setStatus(String(err));
    } finally {
      setBusy(false);
    }
  };

  if (!open) {
    return (
      <button className="secondary" type="button" onClick={() => setOpen(true)} disabled={!heard.trim()}>
        {t.markWrong}
      </button>
    );
  }

  return (
    <div className="flow" style={{ gap: 8, marginTop: 8 }}>
      <div>
        <label>{t.intent}</label>
        <SearchSelect
          value={intent}
          options={withCurrent(intentOptions, intent)}
          onChange={setIntent}
          allowEmpty={false}
          placeholder={t.intent}
        />
      </div>
      <div>
        <label>{t.intendedTarget}</label>
        <SearchSelect
          value={target}
          options={withCurrent(targetOptions, target)}
          onChange={setTarget}
          allowEmpty={false}
          placeholder={t.intendedTarget}
        />
      </div>
      <div className="row">
        <button className="secondary" type="button" disabled={busy || !target} onClick={() => void save()}>
          {t.markWrongSave}
        </button>
        <button className="ghost" type="button" onClick={() => setOpen(false)}>{t.close}</button>
      </div>
      {status ? <p className="muted">{status}</p> : null}
    </div>
  );
}
