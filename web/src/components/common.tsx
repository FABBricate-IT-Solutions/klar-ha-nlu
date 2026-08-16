import type { ReactNode } from "react";

export function Kpi({ value, label, hot }: { value: ReactNode; label: string; hot?: boolean }) {
  return (
    <div className={`card kpi${hot ? " hot" : ""}`}>
      <div className="value">{value}</div>
      <div className="label">{label}</div>
    </div>
  );
}

export function Drawer({ title, children, onClose, closeLabel = "Close" }: { title: string; children: ReactNode; onClose: () => void; closeLabel?: string }) {
  return (
    <>
      <button className="drawer-backdrop" aria-label="close" onClick={onClose} />
      <aside className="drawer" onKeyDown={(ev) => ev.key === "Escape" && onClose()}>
        <div className="row" style={{ justifyContent: "space-between", marginBottom: 18 }}>
          <h2>{title}</h2>
          <button className="ghost" onClick={onClose}>{closeLabel}</button>
        </div>
        {children}
      </aside>
    </>
  );
}

export function Empty({ text, action }: { text: string; action?: ReactNode }) {
  return (
    <div className="card">
      <p className="muted">{text}</p>
      {action}
    </div>
  );
}
