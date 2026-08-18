import { useEffect, useRef, type ReactNode } from "react";

export function Kpi({ value, label, hot }: { value: ReactNode; label: string; hot?: boolean }) {
  return (
    <div className={`card kpi${hot ? " hot" : ""}`}>
      <div className="value">{value}</div>
      <div className="label">{label}</div>
    </div>
  );
}

function focusable(root: HTMLElement): HTMLElement[] {
  return [...root.querySelectorAll<HTMLElement>("button, [href], input, select, textarea, [tabindex]:not([tabindex='-1'])")].filter(
    (node) => !node.hasAttribute("disabled"),
  );
}

export function Drawer({ title, children, onClose, closeLabel = "Close" }: { title: string; children: ReactNode; onClose: () => void; closeLabel?: string }) {
  const panel = useRef<HTMLElement>(null);
  const prior = useRef<HTMLElement | null>(null);
  useEffect(() => {
    prior.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    const root = panel.current;
    focusable(root || document.body)[0]?.focus();
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        onClose();
        return;
      }
      if (event.key !== "Tab" || !root) return;
      const items = focusable(root);
      if (items.length === 0) return;
      const first = items[0];
      const last = items[items.length - 1];
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    };
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("keydown", onKey);
      prior.current?.focus();
    };
  }, [onClose]);
  return (
    <>
      <button className="drawer-backdrop" aria-label={closeLabel} onClick={onClose} />
      <aside ref={panel} className="drawer" role="dialog" aria-modal="true" aria-labelledby="klar-drawer-title">
        <div className="row" style={{ justifyContent: "space-between", marginBottom: 18 }}>
          <h2 id="klar-drawer-title">{title}</h2>
          <button className="ghost" onClick={onClose}>{closeLabel}</button>
        </div>
        {children}
      </aside>
    </>
  );
}

export function Empty({ text, action }: { text: ReactNode; action?: ReactNode }) {
  return (
    <div className="card">
      <p className="muted">{text}</p>
      {action}
    </div>
  );
}
