import { useCallback, useEffect, useState } from "react";
import { api } from "../api";
import { fill, type Messages } from "../i18n";

export type HistoryRow = { hash: string; label: string; saved_at: string };

export function ChangeHistoryCard({
  t,
  kind,
  onRestored,
}: {
  t: Messages;
  kind: "policies" | "settings";
  onRestored?: () => void;
}) {
  const [rows, setRows] = useState<HistoryRow[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

  const load = useCallback(() => {
    const fetchRows = kind === "policies" ? api.policiesHistory : api.settingsHistory;
    fetchRows()
      .then((next) => {
        setRows(Array.isArray(next) ? next : []);
        setError("");
      })
      .catch((err) => setError(String(err)));
  }, [kind]);

  useEffect(() => {
    load();
  }, [load]);

  const rollback = async (hash: string) => {
    setBusy(true);
    try {
      if (kind === "policies") await api.rollbackPolicies(hash);
      else await api.rollbackSettings(hash);
      load();
      onRestored?.();
      setError("");
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  };

  if (rows.length === 0 && !error) return null;

  return (
    <section className="card" style={{ marginTop: 16 }}>
      <h2>{t.changeHistory}</h2>
      <p className="muted">{t.changeHistoryHint}</p>
      {error ? <p className="danger">{error}</p> : null}
      {rows.map((row) => (
        <div className="row" key={row.hash} style={{ marginTop: 8 }}>
          <span className="muted">
            {row.label}
            {row.saved_at ? ` · ${formatStamp(row.saved_at)}` : ""}
          </span>
          <button className="ghost" type="button" disabled={busy} onClick={() => void rollback(row.hash)}>
            {fill(t.changeHistoryRestore, { hash: row.hash.slice(0, 8) })}
          </button>
        </div>
      ))}
    </section>
  );
}

function formatStamp(raw: string): string {
  const secs = Number(raw);
  if (!Number.isFinite(secs) || secs <= 0) return raw;
  try {
    return new Date(secs * 1000).toLocaleString();
  } catch {
    return raw;
  }
}
