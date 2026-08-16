import { useMemo, useState } from "react";
import type { Messages } from "../i18n";
import type { Assignment, Dashboard } from "../types";

export function EntitiesPage({ data, t, onInspect }: { data: Dashboard; t: Messages; onInspect: (row: Assignment) => void }) {
  const [query, setQuery] = useState("");
  const rows = useMemo(() => {
    const q = query.toLowerCase();
    return data.assignment.filter((row) => [row.name, row.entity_id, row.area || "", row.aliases.join(" ")].join(" ").toLowerCase().includes(q));
  }, [data, query]);
  return (
    <div className="page">
      <section className="hero">
        <div>
          <h1>{t.entities}</h1>
          <p className="muted">{data.counts.assist} {t.assistVisible}</p>
        </div>
      </section>
      <label>{t.searchDevice}</label>
      <input value={query} onChange={(ev) => setQuery(ev.target.value)} autoComplete="off" />
      <div className="card" style={{ marginTop: 16 }}>
        <table>
          <thead><tr><th>{t.entities}</th><th>{t.room}</th><th>{t.confidence}</th><th>{t.alias}</th></tr></thead>
          <tbody>
            {rows.map((row) => (
              <tr key={row.entity_id} onClick={() => onInspect(row)}>
                <td>{row.name}<div className="mono">{row.entity_id}</div></td>
                <td>{row.area || "..."}</td>
                <td className={`conf-${row.confidence}`}>{t[row.confidence]}</td>
                <td>{row.aliases.join(", ")}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
