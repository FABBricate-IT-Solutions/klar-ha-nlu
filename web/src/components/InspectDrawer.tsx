import { useEffect, useState } from "react";
import { api } from "../api";
import type { Messages } from "../i18n";
import type { Assignment, Dashboard } from "../types";
import { Drawer } from "./common";

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
    } catch (err) {
      setError(String(err));
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
      <label className="row" style={{ marginTop: 12 }}>
        <input type="checkbox" checked={preferred} onChange={(ev) => setPreferred(ev.target.checked)} />
        <span>{t.preferred}</span>
      </label>
      <label className="row" style={{ marginTop: 12 }}>
        <input type="checkbox" checked={nluIgnore} onChange={(ev) => setNluIgnore(ev.target.checked)} />
        <span>{t.nluIgnore}</span>
      </label>
      <p className="muted">{t.nluIgnoreHint}</p>
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
