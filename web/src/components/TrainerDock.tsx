import { SparklesIcon } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import type { Messages } from "../i18n";
import type { PolicyLane } from "./PolicyPath";
import { TrainerDrawer } from "./TrainerDrawer";

const MIN_W = 300;
const STORE = "klar-lotse-width";

function clampWidth(px: number): number {
  const max = Math.max(MIN_W, Math.round(window.innerWidth * 0.92));
  return Math.min(max, Math.max(MIN_W, Math.round(px)));
}

function readWidth(): number {
  try {
    const raw = Number(window.localStorage.getItem(STORE));
    if (Number.isFinite(raw) && raw >= MIN_W) return clampWidth(raw);
  } catch {
    /* ignore */
  }
  return 420;
}

export function TrainerToggle({
  open,
  onOpenChange,
  t,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  t: Messages;
}) {
  return (
    <Button
      type="button"
      variant={open ? "secondary" : "ghost"}
      aria-pressed={open}
      aria-expanded={open}
      aria-controls="klar-trainer"
      onClick={() => onOpenChange(!open)}
    >
      <SparklesIcon data-icon="inline-start" />
      {t.trainer}
    </Button>
  );
}

export function TrainerDock({
  open,
  onOpenChange,
  t,
  language,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  t: Messages;
  language?: string;
}) {
  const [lane, setLane] = useState<PolicyLane>("house");
  const [status, setStatus] = useState("");
  const [width, setWidth] = useState(readWidth);
  const drag = useRef(false);

  useEffect(() => {
    document.documentElement.style.setProperty("--trainer-width", `${width}px`);
    try {
      window.localStorage.setItem(STORE, String(width));
    } catch {
      /* ignore */
    }
    return () => {
      document.documentElement.style.removeProperty("--trainer-width");
    };
  }, [width]);

  useEffect(() => {
    if (!open) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") onOpenChange(false);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onOpenChange]);

  const move = (clientX: number) => {
    setWidth(clampWidth(window.innerWidth - clientX));
  };

  return (
    <>
      {open ? (
        <button
          type="button"
          className="trainer-backdrop"
          aria-label={t.close}
          onClick={() => onOpenChange(false)}
        />
      ) : null}
      <aside
        className="trainer-dock"
        id="klar-trainer"
        hidden={!open}
        inert={!open}
        aria-hidden={!open}
        aria-label={t.trainer}
      >
        <button
          type="button"
          className="trainer-resize"
          aria-label={t.trainer}
          aria-orientation="vertical"
          aria-valuemin={MIN_W}
          aria-valuenow={width}
          onPointerDown={(event) => {
            drag.current = true;
            event.currentTarget.setPointerCapture(event.pointerId);
          }}
          onPointerMove={(event) => {
            if (drag.current) move(event.clientX);
          }}
          onPointerUp={() => {
            drag.current = false;
          }}
          onKeyDown={(event) => {
            if (event.key === "ArrowLeft") setWidth((prev) => clampWidth(prev + 24));
            if (event.key === "ArrowRight") setWidth((prev) => clampWidth(prev - 24));
          }}
        />
        <TrainerDrawer
          t={t}
          lane={lane}
          language={language}
          onLane={setLane}
          onClose={() => onOpenChange(false)}
          onStatus={setStatus}
        />
        {status ? <p className="muted trainer-dock-status">{status}</p> : null}
      </aside>
    </>
  );
}
