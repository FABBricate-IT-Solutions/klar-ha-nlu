import { useEffect, useState } from "react";
import { api } from "../api";
import type { Messages } from "../i18n";
import { saveFailMessage, toastSaved, toastSaveFailed } from "../saveToast";
import type { Assignment, Dashboard } from "../types";
import { Drawer } from "./common";
import { Checkbox } from "@/components/ui/checkbox";
import { Field, FieldContent, FieldDescription, FieldGroup, FieldLabel } from "@/components/ui/field";

export function InspectDrawer({
  row,
  rooms,
  t,
  onClose,
  onSaved,
  onDismiss,
}: {
  row: Assignment;
  rooms: Dashboard["rooms"];
  t: Messages;
  onClose: () => void;
  onSaved: () => void;
  onDismiss: (row: Assignment) => void;
}) {
  const [aliasDraft, setAliasDraft] = useState(row.aliases.join(", "));
  const [nluIgnore, setNluIgnore] = useState(row.tags.includes("nlu_ignore"));
  const [preferred, setPreferred] = useState(row.tags.includes("preferred"));
  const [area, setArea] = useState(row.area || "");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    setAliasDraft(row.aliases.join(", "));
    setNluIgnore(row.tags.includes("nlu_ignore"));
    setPreferred(row.tags.includes("preferred"));
    setArea(row.area || "");
    setError("");
  }, [row]);

  const persist = async (nextArea = area) => {
    setBusy(true);
    setError("");
    try {
      const aliases = aliasDraft.split(",").map((item) => item.trim()).filter(Boolean);
      await api.tagEntity({
        entity_id: row.entity_id,
        aliases,
        preferred,
        nlu_ignore: nluIgnore,
        area: nextArea || undefined,
      });
      onSaved();
      toastSaved(t);
    } catch (err) {
      toastSaveFailed(t, err);
      setError(saveFailMessage(t, err));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Drawer title={row.name} onClose={onClose} closeLabel={t.close}>
      <p className="mono">{row.entity_id}</p>
      <p className={`conf-${row.confidence}`}>{t[row.confidence]}</p>
      <label htmlFor="klar-inspect-alias">{t.alias}</label>
      <input
        id="klar-inspect-alias"
        value={aliasDraft}
        onChange={(ev) => setAliasDraft(ev.target.value)}
        placeholder={t.searchDevice}
        autoComplete="off"
      />
      <FieldGroup className="mt-3">
        <Field orientation="horizontal">
          <Checkbox
            id="klar-inspect-preferred"
            checked={preferred}
            onCheckedChange={(next) => setPreferred(Boolean(next))}
          />
          <FieldContent className="min-w-0">
            <FieldLabel htmlFor="klar-inspect-preferred" className="max-w-full">
              {t.preferred}
            </FieldLabel>
          </FieldContent>
        </Field>
        <Field orientation="horizontal">
          <Checkbox
            id="klar-inspect-nlu-ignore"
            checked={nluIgnore}
            onCheckedChange={(next) => setNluIgnore(Boolean(next))}
          />
          <FieldContent className="min-w-0">
            <FieldLabel htmlFor="klar-inspect-nlu-ignore" className="max-w-full">
              {t.nluIgnore}
            </FieldLabel>
            <FieldDescription>{t.nluIgnoreHint}</FieldDescription>
          </FieldContent>
        </Field>
      </FieldGroup>
      <label htmlFor="klar-inspect-room">{t.room}</label>
      <select id="klar-inspect-room" value={area} onChange={(ev) => setArea(ev.target.value)}>
        <option value="">{t.otherRoom}</option>
        {rooms.map((room) => (
          <option value={room.area_id} key={room.area_id}>{room.name}</option>
        ))}
      </select>
      {error && <p className="danger">{error}</p>}
      <div className="row" style={{ marginTop: 16 }}>
        <button className="secondary" type="button" onClick={() => persist()} disabled={busy}>{t.save}</button>
        {row.suggested_area && (
          <button
            className="primary"
            type="button"
            disabled={busy}
            onClick={() => persist(row.suggested_area?.area_id || "")}
          >
            {t.accept}: {row.suggested_area.name}
          </button>
        )}
        <button className="ghost" type="button" onClick={() => onDismiss(row)} disabled={busy}>{t.dismiss}</button>
      </div>
    </Drawer>
  );
}
