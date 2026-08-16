import { useEffect, useMemo, useRef, useState } from "react";
import { api } from "./api";
import { Drawer } from "./components/common";
import { dictionaries, initialLocale } from "./i18n";
import { CalibratePage } from "./pages/CalibratePage";
import { CustomPage } from "./pages/CustomPage";
import { DashboardPage } from "./pages/Dashboard";
import { EntitiesPage } from "./pages/EntitiesPage";
import { GraphPage } from "./pages/GraphPage";
import { ParsePage } from "./pages/ParsePage";
import { SettingsPage } from "./pages/SettingsPage";
import type { Assignment, Dashboard, Locale, Settings, Tab, UiState } from "./types";

const tabs: Tab[] = ["dashboard", "graph", "parse", "calibrate", "entities", "custom", "settings"];
const defaultUi: UiState = { tab: "dashboard", locale: "de", dismissed: [], last_apply: [], graph: {} };
const defaultSettings: Settings = { personality: "default", mode: "full", languages: ["de", "en"], support_bundle: false };

export function App() {
  const [ui, setUi] = useState<UiState>(defaultUi);
  const [settings, setSettings] = useState<Settings>(defaultSettings);
  const [dashboard, setDashboard] = useState<Dashboard | null>(null);
  const [inspecting, setInspecting] = useState<Assignment | null>(null);
  const [confirmApply, setConfirmApply] = useState(false);
  const [replayText, setReplayText] = useState("");
  const [error, setError] = useState("");
  const uiLoaded = useRef(false);
  const t = dictionaries[ui.locale] || dictionaries.de;

  const refresh = async () => {
    try {
      const [nextSettings, nextDashboard] = await Promise.all([api.settings(), api.dashboard()]);
      setSettings(nextSettings);
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
        setSettings(nextSettings);
        setUi({ ...defaultUi, ...nextUi, locale });
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
    setTab("parse");
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
  const accept = async (row: Assignment, area = row.suggested_area?.area_id || "") => {
    await api.tagEntity({ entity_id: row.entity_id, aliases: row.aliases, preferred: row.tags.includes("preferred"), area });
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
          <span className={`pill${settings.support_bundle ? " hot" : ""}`}>{settings.support_bundle ? t.bundleOn : t.bundleOff}</span>
        </div>
      </header>
      {error && <div className="page"><div className="card danger">{error}</div></div>}
      {!dashboard && !error && <div className="page"><div className="card hot">{t.loading}</div></div>}
      {dashboard && ui.tab === "dashboard" && <DashboardPage data={dashboard} t={t} onReplay={replay} onApply={() => setConfirmApply(true)} onOpenCalibrate={() => setTab("calibrate")} canApply={applyCandidates.length > 0} />}
      {dashboard && ui.tab === "graph" && <GraphPage data={dashboard} ui={ui} t={t} onUi={setUi} onInspect={setInspecting} />}
      {ui.tab === "parse" && <ParsePage t={t} locale={ui.locale} replayText={replayText} />}
      {dashboard && ui.tab === "calibrate" && <CalibratePage data={dashboard} ui={ui} t={t} onUi={setUi} onRefresh={refresh} onInspect={setInspecting} onApply={() => setConfirmApply(true)} />}
      {dashboard && ui.tab === "entities" && <EntitiesPage data={dashboard} t={t} onInspect={setInspecting} />}
      {ui.tab === "custom" && <CustomPage t={t} />}
      {ui.tab === "settings" && <SettingsPage t={t} settings={settings} onSettings={setSettings} />}

      {inspecting && (
        <Drawer title={inspecting.name} onClose={() => setInspecting(null)} closeLabel={t.close}>
          <p className="mono">{inspecting.entity_id}</p>
          <p className={`conf-${inspecting.confidence}`}>{t[inspecting.confidence]}</p>
          <label>{t.alias}</label>
          <input value={inspecting.aliases.join(", ")} readOnly />
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
