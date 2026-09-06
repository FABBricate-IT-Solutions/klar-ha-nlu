import type { HouseView, RulesView, SettingsView, Tab, UiState } from "./types";

export type Route = {
  tab: Tab;
  house_view?: HouseView;
  rules_view?: RulesView;
  settings_view?: SettingsView;
  entity_id?: string;
};

const houseViews: HouseView[] = ["graph", "entities", "calibrate"];
const rulesViews: RulesView[] = ["routines", "sentences", "policies"];
export const settingsViews: SettingsView[] = ["llm", "voice", "languages", "engine", "backup"];

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

export function asTab(value: string | undefined): Tab {
  return legacyTab[value || ""] || "home";
}

export function asHouseView(value: string | undefined): HouseView {
  return houseViews.includes(value as HouseView) ? (value as HouseView) : "calibrate";
}

export function asRulesView(value: string | undefined): RulesView {
  return rulesViews.includes(value as RulesView) ? (value as RulesView) : "routines";
}

export function asSettingsView(value: string | undefined): SettingsView {
  return settingsViews.includes(value as SettingsView) ? (value as SettingsView) : "llm";
}

export function parseHash(raw: string): Route | null {
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
      return parseSettingsHash(rest);
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

function parseSettingsHash(rest: string[]): Route {
  const sub = rest[0] || "";
  if (settingsViews.includes(sub as SettingsView)) {
    return { tab: "settings", settings_view: sub as SettingsView };
  }
  return { tab: "settings" };
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

export function settingsHash(view: SettingsView | undefined): string {
  return `#/settings/${asSettingsView(view)}`;
}

export function hrefFor(tab: Tab, ui: UiState, entityId?: string): string {
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
      return settingsHash(ui.settings_view);
    default: {
      const _never: never = tab;
      return _never;
    }
  }
}

export function applyRoute(prev: UiState, route: Route): UiState {
  return {
    ...prev,
    tab: route.tab,
    house_view: route.house_view ?? prev.house_view ?? "calibrate",
    rules_view: route.rules_view ?? prev.rules_view ?? "routines",
    settings_view: route.settings_view ?? prev.settings_view ?? "llm",
  };
}
