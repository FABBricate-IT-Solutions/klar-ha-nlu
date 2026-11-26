import { lazy, Suspense, useEffect, useState } from "react";
import { api, download, getToken, setToken, type LanguagePack } from "../api";
import { Guide } from "../components/Guide";
import { LlmSettingsCard } from "../components/LlmSettingsCard";
import { SettingsBackupCard } from "../components/SettingsBackupCard";
import {
  SettingsEngineSection,
  SettingsJournalSection,
  SettingsLanguagesSection,
  SettingsVoiceSection,
} from "../components/SettingsSections";
import { doctorCopy } from "../doctorCopy";
import { type Messages } from "../i18n";
import { sameJson, syncDraftWhenClean } from "../lib/resourceStore";
import { toastSaved, toastSaveFailed } from "../saveToast";
import { settingsViews } from "../routes";
import type { BundleList, Locale, Settings, SettingsView, Theme } from "../types";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";

const DoctorCard = lazy(async () => {
  const mod = await import("../components/DoctorCard");
  return { default: mod.DoctorCard };
});

function readTheme(theme?: Theme): Theme {
  if (theme === "light" || theme === "dark") return theme;
  return document.documentElement.dataset.theme === "light" ? "light" : "dark";
}

function settingsLabel(t: Messages, locale: string, view: SettingsView): string {
  switch (view) {
    case "llm":
      return t.settingsNavLlm;
    case "voice":
      return t.settingsNavVoice;
    case "languages":
      return t.settingsNavLanguages;
    case "engine":
      return t.settingsNavEngine;
    case "doctor":
      return doctorCopy(locale).nav;
    case "backup":
      return t.settingsNavBackup;
    default: {
      const _never: never = view;
      return _never;
    }
  }
}

function settingsHint(t: Messages, locale: string, view: SettingsView): string {
  switch (view) {
    case "llm":
      return t.llmHint;
    case "voice":
      return t.voiceHint;
    case "languages":
      return t.assistLanguagesHint;
    case "engine":
      return t.missHint;
    case "doctor":
      return doctorCopy(locale).hint;
    case "backup":
      return t.journalHint;
    default: {
      const _never: never = view;
      return _never;
    }
  }
}

export function SettingsPage({
  t,
  locale,
  onLocale,
  settings,
  loaded,
  onSettings,
  onTokenApplied,
  onReplayWizard,
  onRestored,
  theme,
  onTheme,
  settingsView,
  onSettingsView,
}: {
  t: Messages;
  locale: Locale;
  onLocale: (locale: Locale) => void;
  settings: Settings;
  /** False until the engine's settings were read; `settings` then holds placeholders only. */
  loaded: boolean;
  /** Persist the saved settings in the app chrome (badges). Draft edits stay local until Save. */
  onSettings: (s: Settings) => void;
  /** The write token changed; everything that failed to load should be read again. */
  onTokenApplied: () => void;
  onReplayWizard?: () => void;
  /** Backup restore finished: reload UI state so the next navigation does not overwrite it. */
  onRestored?: () => void;
  theme?: Theme;
  onTheme?: (theme: Theme) => void;
  settingsView: SettingsView;
  onSettingsView: (view: SettingsView) => void;
}) {
  const [bundle, setBundle] = useState<BundleList | null>(null);
  const [confirmClear, setConfirmClear] = useState(false);
  const [token, setTokenValue] = useState(getToken());
  const [picked, setPicked] = useState<Theme>(() => readTheme(theme));
  const [packs, setPacks] = useState<LanguagePack[]>([]);
  const [llmEpoch, setLlmEpoch] = useState(0);
  const [llmReady, setLlmReady] = useState(false);
  const [saving, setSaving] = useState(false);
  const [draft, setDraft] = useState(settings);
  const [dirty, setDirty] = useState(false);
  const view = settingsView;
  // Derive the shown value so a clean page tracks the saved props without an effect.
  const shown = syncDraftWhenClean(settings, draft, dirty);
  const showDirty = loaded && dirty && !sameJson(shown, settings);
  const refresh = () => api.bundle().then(setBundle).catch(() => undefined);
  const loadExtras = () => {
    refresh();
    // `all=true` so the languages view can show tier badges and filter by the draft switch.
    api.languages(true).then(setPacks).catch(() => undefined);
    api.llmEndpoint().then((next) => setLlmReady(Boolean(next.configured))).catch(() => undefined);
  };
  useEffect(loadExtras, []);
  useEffect(() => {
    if (theme === "light" || theme === "dark") setPicked(theme);
  }, [theme]);
  const editDraft = (next: Settings) => {
    setDraft(next);
    setDirty(true);
  };
  const save = async (next = shown) => {
    if (!loaded) return;
    setSaving(true);
    try {
      const saved = await api.saveSettings(next);
      setDraft(saved);
      setDirty(false);
      onSettings(saved);
      refresh();
      toastSaved(t);
    } catch (err) {
      toastSaveFailed(t, err, "settings");
    } finally {
      setSaving(false);
    }
  };
  const discard = () => {
    setDirty(false);
  };
  const applyToken = () => {
    setToken(token);
    loadExtras();
    onTokenApplied();
  };
  const clear = async () => {
    await api.clearBundle();
    refresh();
    setConfirmClear(false);
  };
  const setTheme = (next: Theme) => {
    setPicked(next);
    document.documentElement.dataset.theme = next;
    document.documentElement.classList.toggle("dark", next !== "light");
    try {
      localStorage.setItem("klar_theme", next);
    } catch {
      /* private mode */
    }
    onTheme?.(next);
  };
  const afterRestore = () => {
    setLlmEpoch((epoch) => epoch + 1);
    // Discard local edits: the restored engine state is the new truth.
    void api.settings().then((next) => {
      setDraft(next);
      setDirty(false);
      onSettings(next);
    }).catch(() => undefined);
    onRestored?.();
  };
  return (
    <div className="page flex min-w-0 flex-col gap-6 overflow-x-hidden">
      <section className="hero">
        <div>
          <h1>{t.settings}</h1>
          <p className="muted">{t.engineHint}</p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button variant="ghost" type="button" onClick={() => onReplayWizard?.()}>
            {t.setupReplay}
          </Button>
          {/* LLM has its own Save; Doctor is read-only (U-24 / 6.6). */}
          {view === "llm" || view === "doctor" ? null : (
            <Button type="button" disabled={saving || !loaded} onClick={() => void save()}>{t.save}</Button>
          )}
        </div>
      </section>
      {showDirty ? (
        <div className="row" role="status" style={{ gap: 12, alignItems: "center" }}>
          <span className="caption">{t.unsavedChanges}</span>
          <Button type="button" size="sm" variant="ghost" disabled={saving} onClick={discard}>{t.discardChanges}</Button>
        </div>
      ) : null}
      <nav className="subnav" aria-label={t.settings}>
        {settingsViews.map((item) => (
          <button
            key={item}
            type="button"
            className={view === item ? "active" : ""}
            aria-current={view === item ? "page" : undefined}
            onClick={() => onSettingsView(item)}
          >
            {settingsLabel(t, locale, item)}
          </button>
        ))}
      </nav>
      <Guide title={t.settingsGuide} steps={[{ id: view, label: settingsLabel(t, locale, view), hint: settingsHint(t, locale, view) }]} />
      <p className="caption" style={{ marginTop: -8 }}>{t.haGlueHint}</p>
      {view === "llm" ? <LlmSettingsCard key={llmEpoch} t={t} /> : null}
      {view === "voice" && loaded ? (
        <SettingsVoiceSection t={t} settings={shown} onSettings={editDraft} locale={locale} llmReady={llmReady} />
      ) : null}
      {view === "languages" && loaded ? (
        <SettingsLanguagesSection
          t={t}
          locale={locale}
          onLocale={onLocale}
          settings={shown}
          onSettings={editDraft}
          packs={packs}
        />
      ) : null}
      {view === "engine" ? (
        <SettingsEngineSection
          t={t}
          settings={shown}
          loaded={loaded}
          onSettings={editDraft}
          token={token}
          onToken={setTokenValue}
          onApplyToken={applyToken}
          theme={picked}
          onTheme={setTheme}
        />
      ) : null}
      {view === "doctor" ? (
        <Suspense fallback={null}>
          <DoctorCard t={t} locale={locale} />
        </Suspense>
      ) : null}
      {view === "backup" ? (
        <div className="flex min-w-0 flex-col gap-4">
          <SettingsBackupCard t={t} onSettings={onSettings} onRestored={afterRestore} />
          {loaded ? <SettingsJournalSection
            t={t}
            settings={shown}
            bundle={bundle}
            onToggle={editDraft}
            onDownloadDataset={() => download("/api/bundle/dataset", "klar-assist-dataset.yaml")}
            onDownloadProtocol={() => download("/api/bundle/protocol", "klar-support-bundle.jsonl")}
            onClear={() => setConfirmClear(true)}
          /> : null}
        </div>
      ) : null}
      <AlertDialog open={confirmClear} onOpenChange={setConfirmClear}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t.clearAll}</AlertDialogTitle>
            <AlertDialogDescription>{t.journalHint}</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t.cancel}</AlertDialogCancel>
            <AlertDialogAction onClick={() => void clear()}>{t.clearAll}</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
