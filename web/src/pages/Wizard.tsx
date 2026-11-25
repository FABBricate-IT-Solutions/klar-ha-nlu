import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { api, type LanguagePack } from "../api";
import { KlarBrand } from "../components/KlarBrand";
import { ChromeStep } from "../components/wizard/ChromeStep";
import { LlmStep } from "../components/wizard/LlmStep";
import { RestStep } from "../components/wizard/RestStep";
import { TutorialStep } from "../components/wizard/TutorialStep";
import { UnitsStep } from "../components/wizard/UnitsStep";
import { VoiceStep } from "../components/wizard/VoiceStep";
import type { Messages } from "../i18n";
import { fillWizard, wizardMessages, type WizardMessages } from "../i18n/wizard";
import { saveFailMessage } from "../saveToast";
import type { Locale, RefineBand, Settings, Theme } from "../types";
import { useLlmForm } from "../useLlmForm";

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
  locale: Locale;
  onLocale: (locale: Locale) => void;
  theme: Theme;
  onTheme: (theme: Theme) => void;
  chrome: Messages;
  settings: Settings;
  onSettings: (next: Settings) => void;
  installPath?: InstallPath;
  leftover?: number;
  entityIds?: readonly string[];
  ingress?: boolean;
  notice?: string;
};

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
  if (value <= 0) return 0;
  if (value >= 5) return 5;
  return value as WizardStep;
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
      return m.chromeTitle;
    case 1:
      return m.unitsTitle;
    case 2:
      return m.missTitle;
    case 3:
      return m.modeTitle;
    case 4:
      return m.restTitle;
    case 5:
      return m.tutorialTitle;
    default: {
      const _never: never = step;
      return _never;
    }
  }
}

export function Wizard({
  open,
  onClose,
  onDone,
  locale,
  onLocale,
  theme,
  onTheme,
  chrome,
  settings,
  onSettings,
  installPath,
  leftover = 0,
  entityIds,
  ingress,
  notice,
}: WizardProps) {
  const root = useRef<HTMLElement>(null);
  const prior = useRef<HTMLElement | null>(null);
  const doneRef = useRef(onDone);
  const closeRef = useRef(onClose);
  const [step, setStep] = useState<WizardStep>(0);
  const scrollRef = useRef<HTMLDivElement>(null);
  const [scrollable, setScrollable] = useState(false);
  const [draft, setDraft] = useState<Settings>(settings);
  const llm = useLlmForm();
  const { load: fillLlm } = llm;
  const llmReady = llm.saved.configured;
  // True only when the engine said at open that no LLM is configured: this run is the first setup.
  const llmWasMissing = useRef(false);
  const [saveError, setSaveError] = useState("");
  const [packs, setPacks] = useState<LanguagePack[]>([]);
  const draftRef = useRef(draft);
  const copy = useMemo(() => wizardMessages(locale), [locale]);
  const path = useMemo(
    () => detectInstallPath({ installPath, entityIds, ingress }),
    [installPath, entityIds, ingress],
  );
  doneRef.current = onDone;
  closeRef.current = onClose;
  draftRef.current = draft;

  /** Saves the draft. A save the engine refuses is not saved: the draft stays in the wizard. */
  const persistSettings = async (next = draftRef.current): Promise<boolean> => {
    try {
      onSettings(await api.saveSettings(next));
      setSaveError("");
      return true;
    } catch (err) {
      setSaveError(saveFailMessage(chrome, err, "settings"));
      return false;
    }
  };

  const persistLlm = async (): Promise<"untouched" | "saved" | "refused"> => {
    if (!llm.baseUrl.trim() && !llm.model.trim() && !llm.apiKey.trim()) return "untouched";
    // The key field shows what is missing: a cloud endpoint at a new address needs its own key.
    if (llm.keyMissing) {
      setSaveError(chrome.llmApiKeyRequired);
      return "refused";
    }
    try {
      fillLlm(await api.saveLlmEndpoint(llm.payload()));
      setSaveError("");
      return "saved";
    } catch (err) {
      setSaveError(saveFailMessage(chrome, err));
      return "refused";
    }
  };

  /** Skip, Escape and the backdrop: leave without writing settings or the LLM endpoint. */
  const dismiss = () => closeRef.current();

  const finish = () => {
    void (async () => {
      if ((await persistLlm()) === "refused" || !(await persistSettings())) return;
      doneRef.current();
      closeRef.current();
    })();
  };

  useEffect(() => {
    if (!open) return;
    setStep(0);
    setDraft(settings);
    setSaveError("");
    llmWasMissing.current = false;
    api.languages(true).then(setPacks).catch(() => undefined);
    api.llmEndpoint().then((next) => {
      fillLlm(next);
      llmWasMissing.current = !next.configured;
    }).catch(() => fillLlm({ configured: false }));
  }, [open, fillLlm]);

  useEffect(() => {
    if (!open) return;
    prior.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    const node = root.current;
    focusable(node || document.body)[0]?.focus();
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        dismiss();
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

  useLayoutEffect(() => {
    const el = scrollRef.current;
    if (!el) return;
    const update = () => setScrollable(el.scrollHeight > el.clientHeight + 2);
    update();
    const ro = new ResizeObserver(update);
    ro.observe(el);
    const child = el.firstElementChild;
    if (child) ro.observe(child);
    return () => ro.disconnect();
  }, [step, open]);

  if (!open) return null;

  const last = step === 5;
  const detected = pathCopy(path, copy);
  const assistLang = draft.languages[0] || locale;

  const goNext = () => {
    void (async () => {
      switch (step) {
        case 0:
        case 1:
        case 3:
        case 4:
          if (!(await persistSettings())) return;
          break;
        case 2: {
          const outcome = await persistLlm();
          if (outcome === "refused") return;
          // Speech refine is switched on once, when this run configures the first LLM. A replay
          // over a configured LLM leaves the operator's refine choice alone.
          if (outcome === "saved" && llmWasMissing.current) {
            const next = { ...draftRef.current, refine_speech: true, refine_bands: ["status"] as RefineBand[] };
            if (!(await persistSettings(next))) return;
            llmWasMissing.current = false;
            setDraft(next);
            draftRef.current = next;
          }
          break;
        }
        case 5:
          break;
        default: {
          const _never: never = step;
          return _never;
        }
      }
      setStep(clampStep(step + 1));
    })();
  };

  const body = (() => {
    switch (step) {
      case 0:
        return (
          <ChromeStep
            copy={copy}
            chrome={chrome}
            locale={locale}
            theme={theme}
            settings={draft}
            packs={packs}
            onLocale={onLocale}
            onTheme={onTheme}
            onSettings={setDraft}
          />
        );
      case 1:
        return <UnitsStep copy={copy} chrome={chrome} settings={draft} onSettings={setDraft} />;
      case 2:
        return <LlmStep copy={copy} chrome={chrome} form={llm} />;
      case 3:
        return (
          <VoiceStep
            copy={copy}
            chrome={chrome}
            settings={draft}
            llmReady={llmReady}
            language={assistLang}
            llm={{ baseUrl: llm.baseUrl, model: llm.model, apiKey: llm.apiKey, provider: llm.provider }}
            onSettings={setDraft}
          />
        );
      case 4:
        return <RestStep copy={copy} chrome={chrome} settings={draft} llmReady={llmReady} onSettings={setDraft} />;
      case 5:
        return <TutorialStep copy={copy} leftover={leftover} pathTitle={detected.title} pathBody={detected.body} />;
      default: {
        const _never: never = step;
        return _never;
      }
    }
  })();

  return (
    <>
      <button type="button" className="wizard-backdrop" aria-label={copy.skip} onClick={dismiss} />
      <div className="wizard-overlay">
        <section
          ref={root}
          className="wizard-panel"
          role="dialog"
          aria-modal="true"
          aria-labelledby="klar-wizard-title"
        >
          <div className="wizard-head">
            <div className="flex min-w-0 items-center gap-2">
              <KlarBrand />
              <p className="pill">{copy.title}</p>
            </div>
            <button type="button" className="ghost" onClick={dismiss}>{copy.skip}</button>
          </div>
          <h1 id="klar-wizard-title">{stepTitle(step, copy)}</h1>
          <p className="caption">{fillWizard(copy.stepOf, { n: String(step + 1), total: "6" })}</p>
          <nav className="wizard-steps" aria-label={copy.title}>
            {STEPS.map((item) => (
              <button
                key={item}
                type="button"
                aria-current={item === step ? "step" : undefined}
                className={item === step ? "primary" : "secondary"}
                onClick={() => setStep(item)}
              >
                {item + 1}
              </button>
            ))}
          </nav>
          <div ref={scrollRef} className={scrollable ? "wizard-scroll is-scrollable" : "wizard-scroll"}>
            <div className="wizard-body">{body}</div>
          </div>
          {saveError ? <p className="caption" role="alert" style={{ color: "var(--danger)" }}>{saveError}</p> : null}
          {notice ? <p className="caption">{notice}</p> : null}
          <div className="wizard-foot">
            <button type="button" className="secondary" onClick={() => setStep(clampStep(step - 1))} disabled={step === 0}>
              {copy.back}
            </button>
            {last ? (
              <button type="button" className="primary" onClick={finish}>{copy.done}</button>
            ) : (
              <button type="button" className="primary" onClick={goNext}>{copy.next}</button>
            )}
          </div>
        </section>
      </div>
    </>
  );
}
