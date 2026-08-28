import { useEffect, useMemo, useRef, useState, type CSSProperties, type ReactNode } from "react";
import { fillWizard, wizardMessages, type WizardMessages } from "../i18n/wizard";

/** Six-step overlay. Skip / Done / Escape / backdrop all call onDone then onClose so Lab is never blocked. */

export type InstallPath = "addon" | "docker" | "binary" | "sample";
export type WizardStep = 0 | 1 | 2 | 3 | 4 | 5;

const STEPS: WizardStep[] = [0, 1, 2, 3, 4, 5];
const SAMPLE_IDS = [
  "vacuum.r2d2",
  "light.schlafzimmer_kugel",
  "climate.better_thermostat_wohnzimmer",
  "light.alle_lichter",
  "switch.kuche_spulmaschine",
  "cover.wohnzimmer_rollo",
];

export type WizardProps = {
  open: boolean;
  onClose: () => void;
  onDone: () => void;
  locale?: string;
  messages?: Partial<WizardMessages>;
  t?: Partial<WizardMessages>;
  installPath?: InstallPath;
  leftover?: number;
  entityIds?: readonly string[];
  ingress?: boolean;
};

const overlay: CSSProperties = {
  position: "fixed",
  inset: 0,
  zIndex: 30,
  display: "grid",
  placeItems: "center",
  padding: 16,
};
const backdrop: CSSProperties = {
  position: "fixed",
  inset: 0,
  background: "rgba(0, 0, 0, .55)",
  zIndex: 29,
  border: 0,
};
const panel: CSSProperties = {
  position: "relative",
  zIndex: 31,
  width: "min(640px, 100%)",
  maxHeight: "min(88vh, 840px)",
  overflow: "auto",
  background: "var(--surface)",
  border: "1px solid var(--line)",
  borderRadius: 0,
  padding: 24,
};
const tap: CSSProperties = { minHeight: 44, minWidth: 44 };
const listReset: CSSProperties = { margin: "12px 0 0", padding: 0, listStyle: "none" };

function focusable(root: HTMLElement): HTMLElement[] {
  return [...root.querySelectorAll<HTMLElement>("button, [href], input, select, textarea, [tabindex]:not([tabindex='-1'])")].filter(
    (node) => !node.hasAttribute("disabled"),
  );
}

function looksLikeIngress(explicit?: boolean): boolean {
  if (explicit) return true;
  if (typeof window === "undefined") return false;
  const path = `${window.location.pathname}${window.location.search}`;
  return path.includes("hassio_ingress") || path.includes("/api/hassio") || path.includes("supervisor/ingress");
}

function looksLikeLoopback(): boolean {
  if (typeof window === "undefined") return false;
  const host = window.location.hostname;
  return host === "localhost" || host === "127.0.0.1" || host === "[::1]" || host === "::1";
}

function looksLikeSample(entityIds?: readonly string[]): boolean {
  if (!entityIds?.length) return false;
  const hits = SAMPLE_IDS.filter((id) => entityIds.includes(id));
  return hits.length >= 3;
}

/** addon = HA ingress; sample = default_home; binary = loopback; else docker. */
export function detectInstallPath(input: {
  installPath?: InstallPath;
  entityIds?: readonly string[];
  ingress?: boolean;
} = {}): InstallPath {
  if (input.installPath) return input.installPath;
  if (looksLikeSample(input.entityIds)) return "sample";
  if (looksLikeIngress(input.ingress)) return "addon";
  if (looksLikeLoopback()) return "binary";
  return "docker";
}

function clampStep(value: number): WizardStep {
  switch (value) {
    case 0:
    case 1:
    case 2:
    case 3:
    case 4:
    case 5:
      return value;
    default:
      return value < 0 ? 0 : 5;
  }
}

function pathCopy(path: InstallPath, m: WizardMessages): { title: string; body: string } {
  switch (path) {
    case "addon":
      return { title: m.pathAddonTitle, body: m.pathAddonBody };
    case "docker":
      return { title: m.pathDockerTitle, body: m.pathDockerBody };
    case "binary":
      return { title: m.pathBinaryTitle, body: m.pathBinaryBody };
    case "sample":
      return { title: m.pathSampleTitle, body: m.pathSampleBody };
    default: {
      const _never: never = path;
      return _never;
    }
  }
}

function stepTitle(step: WizardStep, m: WizardMessages): string {
  switch (step) {
    case 0:
      return m.whatTitle;
    case 1:
      return m.pathTitle;
    case 2:
      return m.modeTitle;
    case 3:
      return m.missTitle;
    case 4:
      return m.toolsTitle;
    case 5:
      return m.phrasesTitle;
    default: {
      const _never: never = step;
      return _never;
    }
  }
}

function Line({ children }: { children: ReactNode }) {
  return <li style={{ marginBottom: 10, color: "var(--text)" }}>{children}</li>;
}

function Block({ title, body, hot, tag }: { title: string; body: string; hot?: boolean; tag?: string }) {
  return (
    <div className={`card${hot ? " hot" : ""}`} style={{ marginTop: 12 }}>
      <div className="row" style={{ justifyContent: "space-between" }}>
        <h2>{title}</h2>
        {tag ? <span className={`pill${hot ? " hot" : ""}`}>{tag}</span> : null}
      </div>
      <p className="muted" style={{ margin: "8px 0 0" }}>{body}</p>
    </div>
  );
}

export function Wizard({
  open,
  onClose,
  onDone,
  locale,
  messages,
  t,
  installPath,
  leftover = 0,
  entityIds,
  ingress,
}: WizardProps) {
  const root = useRef<HTMLElement>(null);
  const prior = useRef<HTMLElement | null>(null);
  const doneRef = useRef(onDone);
  const closeRef = useRef(onClose);
  const [step, setStep] = useState<WizardStep>(0);
  const copy = useMemo(() => wizardMessages(locale, { ...t, ...messages }), [locale, t, messages]);
  const path = useMemo(
    () => detectInstallPath({ installPath, entityIds, ingress }),
    [installPath, entityIds, ingress],
  );
  doneRef.current = onDone;
  closeRef.current = onClose;

  const finish = () => {
    doneRef.current();
    closeRef.current();
  };

  useEffect(() => {
    if (open) setStep(0);
  }, [open]);

  useEffect(() => {
    if (!open) return;
    prior.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    const node = root.current;
    focusable(node || document.body)[0]?.focus();
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        doneRef.current();
        closeRef.current();
        return;
      }
      if (event.key !== "Tab" || !node) return;
      const items = focusable(node);
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
  }, [open]);

  if (!open) return null;

  const last = step === 5;
  const detected = pathCopy(path, copy);

  const body = (() => {
    switch (step) {
      case 0:
        return (
          <>
            <p>{copy.whatLead}</p>
            <ul style={listReset}>
              <Line>{copy.whatLocal}</Line>
              <Line>{copy.whatConsole}</Line>
              <Line>{copy.whatNoLlm}</Line>
            </ul>
          </>
        );
      case 1:
        return (
          <>
            <p>{copy.pathLead}</p>
            <Block title={detected.title} body={detected.body} hot tag={copy.detected} />
            <p className="caption">{copy.pathShared}</p>
          </>
        );
      case 2:
        return (
          <>
            <p>{copy.modeLead}</p>
            <Block title={copy.modeFullTitle} body={copy.modeFullBody} hot tag={copy.recommended} />
            <Block title={copy.modeContextTitle} body={copy.modeContextBody} />
            <Block title={copy.modeNluTitle} body={copy.modeNluBody} />
          </>
        );
      case 3:
        return (
          <>
            <p>{copy.missLead}</p>
            <Block title={copy.missEngineTitle} body={copy.missEngineBody} hot />
            <Block title={copy.missSliceTitle} body={copy.missSliceBody} />
            <Block title={copy.missLlmTitle} body={copy.missLlmBody} />
            <p className="caption" style={{ color: "var(--danger)" }}>{copy.missWarn}</p>
          </>
        );
      case 4:
        return (
          <>
            <p>{copy.toolsLead}</p>
            <ul style={listReset}>
              <Line>{copy.toolsLab}</Line>
              <Line>{copy.toolsMapping}</Line>
              <Line>{copy.toolsPhrases}</Line>
              <Line>{copy.toolsRoutines}</Line>
              <Line>{copy.toolsPolicies}</Line>
            </ul>
          </>
        );
      case 5:
        return (
          <>
            <p>{copy.phrasesLead}</p>
            <table>
              <thead>
                <tr>
                  <th>{copy.phraseSay}</th>
                  <th>{copy.phraseExpect}</th>
                </tr>
              </thead>
              <tbody>
                {copy.phrases.map((row) => (
                  <tr key={row.say}>
                    <td>{row.say}</td>
                    <td className="muted">{row.expect}</td>
                  </tr>
                ))}
              </tbody>
            </table>
            <p className="caption">{copy.phrasesOther}</p>
            {leftover > 0 ? (
              <p className="caption">{fillWizard(copy.phrasesMapping, { count: String(leftover) })}</p>
            ) : null}
            <p className="muted">{copy.phrasesReopen}</p>
          </>
        );
      default: {
        const _never: never = step;
        return _never;
      }
    }
  })();

  return (
    <>
      <button type="button" style={backdrop} aria-label={copy.skip} onClick={finish} />
      <div style={overlay}>
        <section
          ref={root}
          style={panel}
          role="dialog"
          aria-modal="true"
          aria-labelledby="klar-wizard-title"
        >
          <div className="row" style={{ justifyContent: "space-between", marginBottom: 16 }}>
            <p className="pill" style={{ display: "inline-block" }}>{copy.title}</p>
            <button type="button" className="ghost" style={tap} onClick={finish}>{copy.skip}</button>
          </div>
          <h1 id="klar-wizard-title">{stepTitle(step, copy)}</h1>
          <p className="caption">{fillWizard(copy.stepOf, { n: String(step + 1), total: "6" })}</p>
          <nav className="row" style={{ margin: "12px 0 20px", gap: 8 }} aria-label={copy.title}>
            {STEPS.map((item) => (
              <button
                key={item}
                type="button"
                aria-current={item === step ? "step" : undefined}
                className={item === step ? "primary" : "secondary"}
                style={{ ...tap, width: 44, padding: 0 }}
                onClick={() => setStep(item)}
              >
                {item + 1}
              </button>
            ))}
          </nav>
          {body}
          <div className="row" style={{ justifyContent: "space-between", marginTop: 24 }}>
            <button type="button" className="secondary" style={tap} onClick={() => setStep(clampStep(step - 1))} disabled={step === 0}>
              {copy.back}
            </button>
            {last ? (
              <button type="button" className="primary" style={tap} onClick={finish}>{copy.done}</button>
            ) : (
              <button type="button" className="primary" style={tap} onClick={() => setStep(clampStep(step + 1))}>{copy.next}</button>
            )}
          </div>
        </section>
      </div>
    </>
  );
}
