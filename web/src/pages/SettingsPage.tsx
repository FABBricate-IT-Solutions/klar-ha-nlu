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
  onSettings,
  onReplayWizard,
  theme,
  onTheme,
  settingsView,
  onSettingsView,
}: {
  t: Messages;
  locale: Locale;
  onLocale: (locale: Locale) => void;
  settings: Settings;
  onSettings: (s: Settings) => void;
  onReplayWizard?: () => void;
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
  const view = settingsView;
  const refresh = () => api.bundle().then(setBundle).catch(() => undefined);
  useEffect(() => {
    refresh();
  }, []);
  useEffect(() => {
    api.languages().then(setPacks).catch(() => undefined);
    api.llmEndpoint().then((next) => setLlmReady(Boolean(next.configured))).catch(() => undefined);
  }, []);
  useEffect(() => {
    if (theme === "light" || theme === "dark") setPicked(theme);
  }, [theme]);
  const save = async (next = settings) => {
    setToken(token);
    onSettings(await api.saveSettings(next));
    refresh();
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
          <Button type="button" onClick={() => void save()}>{t.save}</Button>
        </div>
      </section>
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
      {view === "voice" ? (
        <SettingsVoiceSection t={t} settings={settings} onSettings={onSettings} locale={locale} llmReady={llmReady} />
      ) : null}
      {view === "languages" ? (
        <SettingsLanguagesSection
          t={t}
          locale={locale}
          onLocale={onLocale}
          settings={settings}
          onSettings={onSettings}
          packs={packs}
        />
      ) : null}
      {view === "engine" ? (
        <SettingsEngineSection
          t={t}
          settings={settings}
          onSettings={onSettings}
          token={token}
          onToken={setTokenValue}
          theme={picked}
          onTheme={setTheme}
        />
      ) : null}
      {view === "backup" ? (
        <div className="flex min-w-0 flex-col gap-4">
          <SettingsBackupCard t={t} onSettings={onSettings} onRestored={() => setLlmEpoch((epoch) => epoch + 1)} />
          <SettingsJournalSection
            t={t}
            settings={settings}
            bundle={bundle}
            onToggle={(next) => void save(next)}
            onDownloadDataset={() => download("/api/bundle/dataset", "klar-assist-dataset.yaml")}
            onDownloadProtocol={() => download("/api/bundle/protocol", "klar-support-bundle.jsonl")}
            onClear={() => setConfirmClear(true)}
          />
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
