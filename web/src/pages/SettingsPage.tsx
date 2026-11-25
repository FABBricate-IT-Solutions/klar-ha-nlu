import { useEffect, useState } from "react";
import { api, download, setToken, type LanguagePack } from "../api";
import { Guide } from "../components/Guide";
import { LlmSettingsCard } from "../components/LlmSettingsCard";
import { SettingsBackupCard } from "../components/SettingsBackupCard";
import {
  SettingsEngineSection,
  SettingsJournalSection,
  SettingsLanguagesSection,
  SettingsVoiceSection,
} from "../components/SettingsSections";
import { type Messages } from "../i18n";
import { sameJson } from "../lib/resourceStore";
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

function readTheme(theme?: Theme): Theme {
  if (theme === "light" || theme === "dark") return theme;
  return document.documentElement.dataset.theme === "light" ? "light" : "dark";
}

function settingsLabel(t: Messages, view: SettingsView): string {
  switch (view) {
    case "llm":
      return t.settingsNavLlm;
    case "voice":
      return t.settingsNavVoice;
    case "languages":
      return t.settingsNavLanguages;
    case "engine":
      return t.settingsNavEngine;
    case "backup":
      return t.settingsNavBackup;
    default: {
      const _never: never = view;
      return _never;
    }
  }
}

function settingsHint(t: Messages, view: SettingsView): string {
  switch (view) {
    case "llm":
      return t.llmHint;
    case "voice":
      return t.voiceHint;
    case "languages":
      return t.assistLanguagesHint;
    case "engine":
      return t.missHint;
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
  const [token, setTokenValue] = useState(localStorage.getItem("klar_token") || "");
  const [picked, setPicked] = useState<Theme>(() => readTheme(theme));
  const [packs, setPacks] = useState<LanguagePack[]>([]);
  const [llmEpoch, setLlmEpoch] = useState(0);
  const [llmReady, setLlmReady] = useState(false);
  const [saving, setSaving] = useState(false);
  const [draft, setDraft] = useState(settings);
  const [draftEpoch, setDraftEpoch] = useState(0);
  const view = settingsView;
  const dirty = loaded && draftEpoch > 0 && !sameJson(draft, settings);
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
  // Keep the draft aligned with the saved settings until the operator edits.
  useEffect(() => {
    if (!dirty) {
      setDraft(settings);
      setDraftEpoch(0);
    }
  }, [settings, dirty]);
  const editDraft = (next: Settings) => {
    setDraft(next);
    setDraftEpoch((epoch) => epoch + 1);
  };
  const save = async (next = draft) => {
    if (!loaded) return;
    setSaving(true);
    try {
      const saved = await api.saveSettings(next);
      setDraft(saved);
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
    setDraft(settings);
    setDraftEpoch(0);
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
          <Button type="button" disabled={saving || !loaded} onClick={() => void save()}>{t.save}</Button>
        </div>
      </section>
      {dirty ? (
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
            {settingsLabel(t, item)}
          </button>
        ))}
      </nav>
      <Guide title={t.settingsGuide} steps={[{ id: view, label: settingsLabel(t, view), hint: settingsHint(t, view) }]} />
      <p className="caption" style={{ marginTop: -8 }}>{t.haGlueHint}</p>
      {view === "llm" ? <LlmSettingsCard key={llmEpoch} t={t} /> : null}
      {view === "voice" && loaded ? (
        <SettingsVoiceSection t={t} settings={draft} onSettings={editDraft} locale={locale} llmReady={llmReady} />
      ) : null}
      {view === "languages" && loaded ? (
        <SettingsLanguagesSection
          t={t}
          locale={locale}
          onLocale={onLocale}
          settings={draft}
          onSettings={editDraft}
          packs={packs}
        />
      ) : null}
      {view === "engine" ? (
        <SettingsEngineSection
          t={t}
          settings={draft}
          loaded={loaded}
          onSettings={editDraft}
          token={token}
          onToken={setTokenValue}
          onApplyToken={applyToken}
          theme={picked}
          onTheme={setTheme}
        />
      ) : null}
      {view === "backup" ? (
        <div className="flex min-w-0 flex-col gap-4">
          <SettingsBackupCard t={t} onSettings={onSettings} onRestored={afterRestore} />
          {loaded ? <SettingsJournalSection
            t={t}
            settings={draft}
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
