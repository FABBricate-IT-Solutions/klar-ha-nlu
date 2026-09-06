import { useEffect, useMemo, useRef, useState } from "react";
import { api, type LanguagePack } from "../api";
import { ChromeStep } from "../components/wizard/ChromeStep";
import { LlmStep } from "../components/wizard/LlmStep";
import { RestStep } from "../components/wizard/RestStep";
import { TutorialStep } from "../components/wizard/TutorialStep";
import { UnitsStep } from "../components/wizard/UnitsStep";
import { VoiceStep } from "../components/wizard/VoiceStep";
import type { Messages } from "../i18n";
import { fillWizard, wizardMessages, type WizardMessages } from "../i18n/wizard";
import type { Locale, Settings, Theme } from "../types";

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
}: WizardProps) {
  const root = useRef<HTMLElement>(null);
  const prior = useRef<HTMLElement | null>(null);
  const doneRef = useRef(onDone);
  const closeRef = useRef(onClose);
  const [step, setStep] = useState<WizardStep>(0);
  const [draft, setDraft] = useState<Settings>(settings);
  const [llmUrl, setLlmUrl] = useState("");
  const [llmModel, setLlmModel] = useState("");
  const [llmKey, setLlmKey] = useState("");
  const [llmThinking, setLlmThinking] = useState(false);
  const [llmReady, setLlmReady] = useState(false);
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

  const persistSettings = async () => {
    const next = draftRef.current;
    try {
      onSettings(await api.saveSettings(next));
    } catch {
      onSettings(next);
    }
  };

  const persistLlm = async (): Promise<boolean> => {
    if (!llmUrl.trim() && !llmModel.trim() && !llmKey.trim()) {
      return llmReady;
    }
    try {
      await api.saveLlmEndpoint({
        base_url: llmUrl,
        model: llmModel,
        ...(llmKey.trim() ? { api_key: llmKey.trim() } : {}),
        configured: true,
        enable_thinking: llmThinking,
      });
      setLlmReady(true);
      return true;
    } catch {
      return llmReady;
    }
  };

  const finish = () => {
    void persistLlm().finally(() => {
      void persistSettings().finally(() => {
        doneRef.current();
        closeRef.current();
      });
    });
  };

  useEffect(() => {
    if (!open) return;
    setStep(0);
    setDraft(settings);
    setLlmKey("");
    api.languages().then(setPacks).catch(() => undefined);
    api.llmEndpoint().then((next) => {
      setLlmUrl(next.base_url || "");
      setLlmModel(next.model || "");
      setLlmThinking(Boolean(next.enable_thinking));
      setLlmReady(Boolean(next.configured));
    }).catch(() => {
      setLlmUrl("");
      setLlmModel("");
      setLlmThinking(false);
      setLlmReady(false);
    });
  }, [open]);

  useEffect(() => {
    if (!open) return;
    prior.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    const node = root.current;
    focusable(node || document.body)[0]?.focus();
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        finish();
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
  const assistLang = draft.languages[0] || locale;

  const goNext = () => {
    void (async () => {
      if (step === 0 || step === 1 || step === 3 || step === 4) await persistSettings();
      if (step === 2) {
        const ready = await persistLlm();
        if (ready) {
          const next = { ...draftRef.current, refine_speech: true };
          setDraft(next);
          draftRef.current = next;
          try {
            onSettings(await api.saveSettings(next));
          } catch {
            onSettings(next);
          }
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
        return (
          <LlmStep
            copy={copy}
            chrome={chrome}
            url={llmUrl}
            model={llmModel}
            apiKey={llmKey}
            thinking={llmThinking}
            onUrl={setLlmUrl}
            onModel={setLlmModel}
            onKey={setLlmKey}
            onThinking={setLlmThinking}
          />
        );
      case 3:
        return (
          <VoiceStep
            copy={copy}
            chrome={chrome}
            settings={draft}
            llmReady={llmReady}
            language={assistLang}
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
      <button type="button" className="wizard-backdrop" aria-label={copy.skip} onClick={finish} />
      <div className="wizard-overlay">
        <section
          ref={root}
          className="wizard-panel"
          role="dialog"
          aria-modal="true"
          aria-labelledby="klar-wizard-title"
        >
          <div className="wizard-head">
            <p className="pill">{copy.title}</p>
            <button type="button" className="ghost" onClick={finish}>{copy.skip}</button>
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
          {body}
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
