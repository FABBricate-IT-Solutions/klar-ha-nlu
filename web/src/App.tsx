import { useEffect, useMemo, useRef, useState } from "react";
import { api } from "./api";
import { Drawer } from "./components/common";
import { dictionaries, initialLocale } from "./i18n";
import { ConversationsPage } from "./pages/ConversationsPage";
import { DashboardPage } from "./pages/Dashboard";
import { HousePage } from "./pages/HousePage";
import { ParsePage } from "./pages/ParsePage";
import { RulesPage } from "./pages/RulesPage";
import { SettingsPage } from "./pages/SettingsPage";
import type { Assignment, Dashboard, Locale, Settings, Tab, UiState } from "./types";

const tabs: Tab[] = ["home", "conversations", "rules", "house", "lab", "settings"];
const legacyTab: Record<string, Tab> = {
  dashboard: "home",
  graph: "house",
  parse: "lab",
  calibrate: "house",
  entities: "house",
  custom: "rules",
  settings: "settings",
  home: "home",
  conversations: "conversations",
  rules: "rules",
  house: "house",
  lab: "lab",
};
const defaultUi: UiState = { tab: "home", locale: "de", dismissed: [], last_apply: [], graph: {} };
const defaultSettings: Settings = {
  personality: "default",
  mode: "full",
  languages: [],
  support_bundle: false,
  support_bundle_raw_text: false,
  confirm_risky_actions: true,
  semantic_adapters: false,
  nlu_rag: false,
};

function asTab(value: string | undefined): Tab {
  return legacyTab[value || ""] || "home";
}

export function App() {
  const [ui, setUi] = useState<UiState>(defaultUi);
  const [settings, setSettings] = useState<Settings>(defaultSettings);
  const [dashboard, setDashboard] = useState<Dashboard | null>(null);
  const [inspecting, setInspecting] = useState<Assignment | null>(null);
  const [aliasDraft, setAliasDraft] = useState("");
  const [nluIgnore, setNluIgnore] = useState(false);
  const [confirmApply, setConfirmApply] = useState(false);
  const [replayText, setReplayText] = useState("");
  const [error, setError] = useState("");
  const uiLoaded = useRef(false);
  const t = dictionaries[ui.locale] || dictionaries.de;

  const refresh = async () => {
    try {
      const [nextSettings, nextDashboard] = await Promise.all([api.settings(), api.dashboard()]);
      setSettings({ ...defaultSettings, ...nextSettings });
      setDashboard(nextDashboard);
      setError("");
    } catch (err) {
      setError(String(err));
    }
  };

  useEffect(() => {
    (async () => {
      try {
        const [nextSettings, nextUi, nextDashboard] = await Promise.all([api.settings(), api.ui(), api.dashboard()]);
        const locale = initialLocale(nextUi.locale, nextSettings.languages);
        setSettings({ ...defaultSettings, ...nextSettings });
        setUi({ ...defaultUi, ...nextUi, locale, tab: asTab(nextUi.tab) });
        setDashboard(nextDashboard);
        uiLoaded.current = true;
      } catch (err) {
        setError(String(err));
      }
    })();
  }, []);

  useEffect(() => {
    if (!uiLoaded.current) return;
    const timer = window.setTimeout(() => api.saveUi(ui).catch(() => undefined), 350);
    return () => window.clearTimeout(timer);
  }, [ui]);

  const applyCandidates = useMemo(
    () => dashboard?.assignment.filter((row) => (row.suggested_area?.score || 0) >= 3 && row.area !== row.suggested_area?.area_id) || [],
    [dashboard],
  );

  const setTab = (tab: Tab) => setUi((prev) => ({ ...prev, tab }));
  const setLocale = (locale: Locale) => setUi((prev) => ({ ...prev, locale }));
  const replay = (text: string) => {
    setReplayText(text);
    setTab("lab");
  };
  const apply = async () => {
    const out = await api.applySuggestions();
    setUi((prev) => ({ ...prev, last_apply: out.rows }));
    setConfirmApply(false);
    refresh();
  };
  const undo = async () => {
    await api.undoApply();
    setUi((prev) => ({ ...prev, last_apply: [] }));
    refresh();
  };
  const openInspect = (row: Assignment) => {
    setAliasDraft(row.aliases.join(", "));
    setNluIgnore(row.tags.includes("nlu_ignore"));
    setInspecting(row);
  };
  const saveAlias = async () => {
    if (!inspecting) return;
    const aliases = aliasDraft.split(",").map((item) => item.trim()).filter(Boolean);
    await api.tagEntity({
      entity_id: inspecting.entity_id,
      aliases,
      preferred: inspecting.tags.includes("preferred"),
      nlu_ignore: nluIgnore,
      area: inspecting.area || undefined,
    });
    setInspecting(null);
    refresh();
  };
  const accept = async (row: Assignment, area = row.suggested_area?.area_id || "") => {
    const aliases = aliasDraft.split(",").map((item) => item.trim()).filter(Boolean);
    await api.tagEntity({ entity_id: row.entity_id, aliases: aliases.length ? aliases : row.aliases, preferred: row.tags.includes("preferred"), nlu_ignore: nluIgnore, area });
    setInspecting(null);
    refresh();
  };
  const dismiss = (row: Assignment) => {
    setUi((prev) => ({ ...prev, dismissed: [...new Set([...prev.dismissed, row.entity_id])] }));
    setInspecting(null);
  };

  return (
    <div className="app-shell">
      <header className="topbar">
        <div className="brand">Klar</div>
        <nav className="nav">
          {tabs.map((tab) => <button key={tab} className={ui.tab === tab ? "active" : ""} onClick={() => setTab(tab)}>{t[tab]}</button>)}
        </nav>
        <div className="status">
          <button className="ghost" onClick={() => setLocale(ui.locale === "de" ? "en" : "de")}>{ui.locale.toUpperCase()}</button>
          <span className={`pill${dashboard?.counts.leftover ? " hot" : ""}`}>{dashboard?.counts.leftover ?? 0} {t.open}</span>
          <span className={`pill${settings.nlu_rag ? " hot" : ""}`}>{settings.nlu_rag ? t.ragMode : t.chatMode}</span>
        </div>
      </header>
      {error && <div className="page"><div className="card danger">{error}</div></div>}
      {!dashboard && !error && <div className="page"><div className="card">{t.loading}</div></div>}
      {dashboard && ui.tab === "home" && <DashboardPage data={dashboard} t={t} locale={ui.locale} onReplay={replay} onApply={() => setConfirmApply(true)} onOpenCalibrate={() => setTab("house")} canApply={applyCandidates.length > 0} />}
      {ui.tab === "conversations" && <ConversationsPage t={t} onReplay={replay} />}
      {ui.tab === "rules" && <RulesPage t={t} locale={ui.locale} personality={settings.personality} languages={settings.languages} />}
      {dashboard && ui.tab === "house" && (
        <HousePage data={dashboard} ui={ui} t={t} onUi={setUi} onInspect={openInspect} onRefresh={refresh} onApply={() => setConfirmApply(true)} />
      )}
      {ui.tab === "lab" && <ParsePage t={t} locale={ui.locale} replayText={replayText} nluRag={settings.nlu_rag} rooms={dashboard?.rooms || []} />}
      {ui.tab === "settings" && (
        <SettingsPage
          t={t}
          settings={settings}
          onSettings={(next) => {
            setSettings(next);
            setUi((prev) => ({ ...prev, locale: initialLocale(prev.locale, next.languages) }));
          }}
        />
      )}

      {inspecting && (
        <Drawer title={inspecting.name} onClose={() => setInspecting(null)} closeLabel={t.close}>
          <p className="mono">{inspecting.entity_id}</p>
          <p className={`conf-${inspecting.confidence}`}>{t[inspecting.confidence]}</p>
          <label>{t.alias}</label>
          <input value={aliasDraft} onChange={(ev) => setAliasDraft(ev.target.value)} placeholder={t.searchDevice} />
          <label className="row" style={{ marginTop: 12 }}>
            <input type="checkbox" checked={nluIgnore} onChange={(ev) => setNluIgnore(ev.target.checked)} />
            <span>{t.nluIgnore}</span>
          </label>
          <p className="muted">{t.nluIgnoreHint}</p>
          <button className="secondary" onClick={saveAlias}>{t.save}</button>
          <label>{t.room}</label>
          <p>{inspecting.area || "..."}</p>
          {inspecting.suggested_area && <button className="primary" onClick={() => accept(inspecting)}>{t.accept}: {inspecting.suggested_area.name}</button>}
          <button className="ghost" onClick={() => dismiss(inspecting)}>{t.dismiss}</button>
        </Drawer>
      )}

      {confirmApply && (
        <Drawer title={t.confirmApply} onClose={() => setConfirmApply(false)} closeLabel={t.close}>
          {applyCandidates.map((row) => <p key={row.entity_id}>{row.name} → {row.suggested_area?.name}</p>)}
          <div className="row">
            <button className="primary" onClick={apply}>{t.apply}</button>
            <button className="secondary" onClick={() => setConfirmApply(false)}>{t.cancel}</button>
          </div>
          {ui.last_apply.length > 0 && <button className="ghost" onClick={undo}>{t.undo}</button>}
        </Drawer>
      )}
    </div>
  );
}
