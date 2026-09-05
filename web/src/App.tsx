import { FlaskConicalIcon, HomeIcon, HouseIcon, MenuIcon, MessageSquareIcon, SettingsIcon, ShieldIcon } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { api } from "./api";
import { Drawer } from "./components/common";
import { TEACH_HEARD_KEY } from "./components/TeachFromMiss";
import { assistParseLanguage, chromeLocale, dictionaries, isRtl } from "./i18n";
import { ConversationsPage } from "./pages/ConversationsPage";
import { DashboardPage } from "./pages/Dashboard";
import { HousePage } from "./pages/HousePage";
import { ParsePage } from "./pages/ParsePage";
import { RulesPage } from "./pages/RulesPage";
import { SettingsPage } from "./pages/SettingsPage";
import { Wizard } from "./pages/Wizard";
import type { ConversationTurn, Dashboard, HouseView, RulesView, Settings, Tab, Theme, UiState } from "./types";
import { Badge } from "@/components/ui/badge";
import { Button, buttonVariants } from "@/components/ui/button";
import { Sheet, SheetContent, SheetHeader, SheetTitle } from "@/components/ui/sheet";
import { Toaster } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { cn } from "cn";

const tabIcons: Record<Tab, typeof HomeIcon> = {
  home: HomeIcon,
  conversations: MessageSquareIcon,
  rules: ShieldIcon,
  house: HouseIcon,
  lab: FlaskConicalIcon,
  settings: SettingsIcon,
};

const railTabs: Tab[] = ["home", "conversations", "rules", "house"];
const utilTabs: Tab[] = ["lab", "settings"];
const houseViews: HouseView[] = ["graph", "entities", "calibrate"];
const rulesViews: RulesView[] = ["routines", "sentences", "policies"];
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
const defaultUi: UiState = {
  tab: "home",
  locale: "",
  locale_set: false,
  dismissed: [],
  last_apply: [],
  graph: {},
  wizard_done: false,
  house_view: "calibrate",
  rules_view: "routines",
  theme: "dark",
};
const defaultSettings: Settings = {
  personality: "default",
  mode: "full",
  languages: [],
  support_bundle: false,
  support_bundle_raw_text: false,
  confirm_risky_actions: true,
  semantic_adapters: false,
  nlu_rag: false,
  refine_speech: false,
  calendar_llm: false,
  quiet_ack: false,
  allow_llm_tools: false,
  fallback_llm: false,
  extra_prompt: "",
};

type Route = {
  tab: Tab;
  house_view?: HouseView;
  rules_view?: RulesView;
  entity_id?: string;
};

function asTab(value: string | undefined): Tab {
  return legacyTab[value || ""] || "home";
}

function asHouseView(value: string | undefined): HouseView {
  return houseViews.includes(value as HouseView) ? (value as HouseView) : "calibrate";
}

function asRulesView(value: string | undefined): RulesView {
  return rulesViews.includes(value as RulesView) ? (value as RulesView) : "routines";
}

function asTheme(value: string | undefined): Theme {
  return value === "light" ? "light" : "dark";
}

function parseHash(raw: string): Route | null {
  if (!raw || raw === "#") return null;
  const parts = raw.replace(/^#/, "").replace(/^\//, "").split("/").filter(Boolean);
  if (parts.length === 0) return { tab: "home" };
  const head = parts[0] || "";
  const rest = parts.slice(1);
  switch (head) {
    case "dashboard":
    case "home":
      return { tab: "home" };
    case "conversations":
      return { tab: "conversations" };
    case "lab":
    case "parse":
      return { tab: "lab" };
    case "settings":
      return { tab: "settings" };
    case "custom":
      return { tab: "rules", rules_view: "sentences" };
    case "rules":
      return parseRulesHash(rest);
    case "house":
      return parseHouseHash(rest);
    case "graph":
      return { tab: "house", house_view: "graph" };
    case "calibrate":
      return { tab: "house", house_view: "calibrate" };
    case "entities":
      return { tab: "house", house_view: "entities" };
    default:
      return { tab: asTab(head) };
  }
}

function parseRulesHash(rest: string[]): Route {
  const sub = rest[0] || "";
  if (sub === "sentences" || sub === "phrases") return { tab: "rules", rules_view: "sentences" };
  if (sub === "policies") return { tab: "rules", rules_view: "policies" };
  if (sub === "routines") return { tab: "rules", rules_view: "routines" };
  return { tab: "rules" };
}

function parseHouseHash(rest: string[]): Route {
  const sub = rest[0] || "";
  if (sub === "mapping" || sub === "calibrate") return { tab: "house", house_view: "calibrate" };
  if (sub === "graph") return { tab: "house", house_view: "graph" };
  if (sub === "entities") return { tab: "house", house_view: "entities" };
  if (sub === "devices") {
    const id = rest.slice(1).map((part) => decodeURIComponent(part)).join("/");
    return { tab: "house", house_view: "entities", entity_id: id || undefined };
  }
  return { tab: "house" };
}

function houseHash(view: HouseView | undefined, entityId?: string): string {
  if (entityId) return `#/house/devices/${encodeURIComponent(entityId)}`;
  const current: HouseView = view || "calibrate";
  switch (current) {
    case "calibrate":
      return "#/house/mapping";
    case "entities":
      return "#/house/devices";
    case "graph":
      return "#/house/graph";
    default: {
      const _never: never = current;
      return _never;
    }
  }
}

function rulesHash(view: RulesView | undefined): string {
  const current: RulesView = view || "routines";
  switch (current) {
    case "routines":
      return "#/rules/routines";
    case "sentences":
      return "#/rules/sentences";
    case "policies":
      return "#/rules/policies";
    default: {
      const _never: never = current;
      return _never;
    }
  }
}

function hrefFor(tab: Tab, ui: UiState, entityId?: string): string {
  switch (tab) {
    case "home":
      return "#/";
    case "conversations":
      return "#/conversations";
    case "rules":
      return rulesHash(ui.rules_view);
    case "house":
      return houseHash(ui.house_view, entityId);
    case "lab":
      return "#/lab";
    case "settings":
      return "#/settings";
    default: {
      const _never: never = tab;
      return _never;
    }
  }
}

function applyRoute(prev: UiState, route: Route): UiState {
  return {
    ...prev,
    tab: route.tab,
    house_view: route.house_view ?? prev.house_view ?? "calibrate",
    rules_view: route.rules_view ?? prev.rules_view ?? "routines",
  };
}

export function App() {
  const [ui, setUi] = useState<UiState>(defaultUi);
  const [settings, setSettings] = useState<Settings>(defaultSettings);
  const [dashboard, setDashboard] = useState<Dashboard | null>(null);
  const [journal, setJournal] = useState<ConversationTurn[] | null>(null);
  const [confirmApply, setConfirmApply] = useState(false);
  const [replayText, setReplayText] = useState("");
  const [error, setError] = useState("");
  const uiLoaded = useRef(false);
  const [inspectId, setInspectId] = useState("");
  const [booted, setBooted] = useState(false);
  const [navOpen, setNavOpen] = useState(false);
  const locale = chromeLocale(ui.locale);
  const t = dictionaries[locale] || dictionaries.en;
  const theme = ui.theme || "dark";

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
        const [nextSettings, nextUi, nextDashboard] = await Promise.all([
          api.settings(),
          api.ui(),
          api.dashboard(),
        ]);
        const next = { ...defaultSettings, ...nextSettings };
        const route = parseHash(window.location.hash);
        setSettings(next);
        setUi(applyRoute({
          ...defaultUi,
          ...nextUi,
          locale: chromeLocale(nextUi.locale),
          locale_set: Boolean(nextUi.locale_set),
          tab: asTab(nextUi.tab),
          house_view: asHouseView(nextUi.house_view),
          rules_view: asRulesView(nextUi.rules_view),
          theme: asTheme(nextUi.theme),
          wizard_done: Boolean(nextUi.wizard_done),
        }, route || { tab: asTab(nextUi.tab) }));
        if (route?.entity_id) setInspectId(route.entity_id);
        setDashboard(nextDashboard);
        api.conversations().then(setJournal).catch(() => undefined);
        uiLoaded.current = true;
        setBooted(true);
      } catch (err) {
        setError(String(err));
        setBooted(true);
      }
    })();
  }, []);

  useEffect(() => {
    if (!uiLoaded.current) return;
    const timer = window.setTimeout(() => api.saveUi({
      ...ui,
      locale: ui.locale_set ? locale : "",
      locale_set: Boolean(ui.locale_set),
    }).catch(() => undefined), 350);
    return () => window.clearTimeout(timer);
  }, [ui, locale]);

  useEffect(() => {
    document.documentElement.lang = locale;
    document.documentElement.dir = isRtl(locale) ? "rtl" : "ltr";
    document.documentElement.dataset.theme = theme;
    document.documentElement.classList.toggle("dark", theme !== "light");
  }, [locale, theme]);

  useEffect(() => {
    if (!uiLoaded.current) return;
    const next = hrefFor(ui.tab, ui, ui.tab === "house" ? (inspectId || undefined) : undefined);
    if (window.location.hash === next) return;
    history.replaceState(null, "", `${window.location.pathname}${window.location.search}${next}`);
  }, [ui.tab, ui.house_view, ui.rules_view, inspectId]);

  useEffect(() => {
    const onHash = () => {
      const route = parseHash(window.location.hash);
      if (!route) return;
      setUi((prev) => applyRoute(prev, route));
      setInspectId(route.entity_id || "");
    };
    window.addEventListener("hashchange", onHash);
    return () => window.removeEventListener("hashchange", onHash);
  }, []);

  const applyCandidates = useMemo(
    () => dashboard?.assignment.filter((row) => (row.suggested_area?.score || 0) >= 3 && row.area !== row.suggested_area?.area_id) || [],
    [dashboard],
  );

  const go = (tab: Tab, extra: Partial<Pick<UiState, "house_view" | "rules_view">> = {}) => {
    setUi((prev) => ({ ...prev, tab, ...extra }));
  };
  const teach = (heard: string) => {
    const phrase = heard.trim();
    if (phrase) sessionStorage.setItem(TEACH_HEARD_KEY, phrase);
    go("rules", { rules_view: "sentences" });
  };
  const replay = (text: string) => {
    setReplayText(text);
    go("lab");
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
  const finishWizard = () => {
    setUi((prev) => {
      const next = { ...prev, wizard_done: true };
      api.saveUi({
        ...next,
        locale: next.locale_set ? locale : "",
        locale_set: Boolean(next.locale_set),
      }).catch(() => undefined);
      return next;
    });
  };
  const replayWizard = () => setUi((prev) => ({ ...prev, wizard_done: false }));
  const persistHouse = (next: UiState) => {
    setUi((prev) => ({
      ...prev,
      ...next,
      tab: prev.tab,
      locale: prev.locale,
      wizard_done: prev.wizard_done,
      house_view: asHouseView(next.house_view ?? prev.house_view),
      rules_view: prev.rules_view,
      theme: prev.theme,
    }));
  };

  const link = (tab: Tab, full = true) => {
    const Icon = tabIcons[tab];
    const active = ui.tab === tab;
    return (
      <a
        key={tab}
        href={hrefFor(tab, ui)}
        className={cn(buttonVariants({ variant: active ? "secondary" : "ghost" }), full && "w-full justify-start")}
        aria-current={active ? "page" : undefined}
        onClick={() => setNavOpen(false)}
      >
        <Icon data-icon="inline-start" />
        {t[tab]}
      </a>
    );
  };

  const navTabs: Tab[] = [...railTabs, ...utilTabs];

  return (
    <TooltipProvider>
    <div className="app-shell" data-theme={theme}>
      <Sheet open={navOpen} onOpenChange={setNavOpen}>
        <SheetContent side="left" id="klar-nav" className="bg-background text-foreground w-64 p-0">
          <SheetHeader>
            <SheetTitle className="brand px-4 pt-2">Klar</SheetTitle>
          </SheetHeader>
          <nav className="flex flex-col gap-1 px-3 pb-4" aria-label="Klar">
            {navTabs.map((tab) => link(tab))}
          </nav>
        </SheetContent>
      </Sheet>
      <div className="app-main">
        <header className="topbar">
          <div className="flex items-center gap-2">
            <Button
              type="button"
              variant="ghost"
              size="icon"
              aria-label={t.menu}
              aria-expanded={navOpen}
              aria-controls="klar-nav"
              onClick={() => setNavOpen(true)}
            >
              <MenuIcon />
            </Button>
            <div className="brand">Klar</div>
          </div>
          <div className="status">
            <Badge variant={dashboard?.counts.leftover ? "default" : "outline"}>{dashboard?.counts.leftover ?? 0} {t.open}</Badge>
            <Badge variant={settings.nlu_rag ? "default" : "outline"}>{settings.nlu_rag ? t.ragMode : t.chatMode}</Badge>
          </div>
        </header>
        {error && <div className="page"><div className="card danger">{error}</div></div>}
        {!dashboard && !error && <div className="page"><div className="card">{t.loading}</div></div>}
        {dashboard && ui.tab === "home" && (
          <DashboardPage
            data={dashboard}
            t={t}
            locale={locale}
            dismissed={ui.dismissed}
            onReplay={replay}
            onApply={() => setConfirmApply(true)}
            onOpenCalibrate={() => go("house", { house_view: "calibrate" })}
            canApply={applyCandidates.length > 0}
            onTeach={teach}
            lastTurn={journal ? journal.at(-1) ?? null : undefined}
            parseLanguage={assistParseLanguage(settings.languages, locale)}
          />
        )}
        {ui.tab === "conversations" && <ConversationsPage t={t} locale={locale} onReplay={replay} onTeach={teach} />}
        {ui.tab === "rules" && (
          <RulesPage
            t={t}
            locale={locale}
            personality={settings.personality}
            languages={settings.languages}
            rulesView={asRulesView(ui.rules_view)}
            onRulesView={(view) => setUi((prev) => ({ ...prev, rules_view: view }))}
          />
        )}
        {dashboard && ui.tab === "house" && (
          <HousePage
            data={dashboard}
            ui={ui}
            t={t}
            onUi={persistHouse}
            onInspect={(row) => setInspectId(row?.entity_id || "")}
            onRefresh={refresh}
            onApply={() => setConfirmApply(true)}
            houseView={asHouseView(ui.house_view)}
            onHouseView={(view) => go("house", { house_view: view })}
            inspectId={inspectId || undefined}
          />
        )}
        {ui.tab === "lab" && (
          <ParsePage
            t={t}
            parseLanguage={assistParseLanguage(settings.languages, locale)}
            replayText={replayText}
            settings={settings}
            rooms={dashboard?.rooms || []}
          />
        )}
        {ui.tab === "settings" && (
          <SettingsPage
            t={t}
            locale={locale}
            onLocale={(next) => setUi((prev) => ({ ...prev, locale: next, locale_set: true }))}
            settings={settings}
            onSettings={setSettings}
            onReplayWizard={replayWizard}
            theme={theme}
            onTheme={(next) => setUi((prev) => ({ ...prev, theme: next }))}
          />
        )}
      </div>

      {booted && !ui.wizard_done && (
        <Wizard
          open
          locale={locale}
          leftover={dashboard?.counts.leftover ?? 0}
          entityIds={dashboard?.assignment.map((row) => row.entity_id)}
          chrome={t}
          settings={settings}
          onSettings={setSettings}
          onDone={finishWizard}
          onClose={() => undefined}
        />
      )}

      {confirmApply && (
        <Drawer title={t.confirmApply} onClose={() => setConfirmApply(false)} closeLabel={t.close}>
          {applyCandidates.map((row) => <p key={row.entity_id}>{row.name} → {row.suggested_area?.name}</p>)}
          <div className="flex flex-wrap gap-2">
            <Button type="button" onClick={() => void apply()}>{t.apply}</Button>
            <Button variant="outline" type="button" onClick={() => setConfirmApply(false)}>{t.cancel}</Button>
          </div>
          {ui.last_apply.length > 0 && <Button variant="ghost" type="button" onClick={() => void undo()}>{t.undo}</Button>}
        </Drawer>
      )}
    </div>
    <Toaster theme={theme} />
    </TooltipProvider>
  );
}
