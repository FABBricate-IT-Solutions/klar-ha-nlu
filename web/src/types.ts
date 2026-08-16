export type Locale = "de" | "en";
export type Tab = "dashboard" | "graph" | "parse" | "calibrate" | "entities" | "custom" | "settings";
export type Confidence = "high" | "medium" | "low";

export type Settings = {
  personality: string;
  mode: "full" | "context_only";
  languages: string[];
  support_bundle: boolean;
};

export type Entity = {
  entity_id: string;
  name: string;
  domain: string;
  area: string | null;
  aliases: string[];
  tags: string[];
};

export type Area = {
  area_id: string;
  name: string;
  aliases: string[];
};

export type Slot = { name: string; value: string };
export type Intent = { name: string; slots: Slot[] };

export type ParseResult = {
  text: string;
  intents: Intent[];
  speech: string;
  clarify: boolean;
  conversation_id: string;
  chat: boolean;
  briefing: boolean;
  personality: string;
};

export type Suggestion = {
  area_id: string;
  name: string;
  score: number;
  reasons: string[];
};

export type Assignment = Entity & {
  confidence: Confidence;
  suggested_area?: Suggestion | null;
  reasons: string[];
};

export type Dashboard = {
  counts: {
    all: number;
    assist: number;
    rooms: number;
    leftover: number;
    high: number;
    medium: number;
    low: number;
    bundle: number;
  };
  coverage: { all: number; assist: number; high: number; leftover: number };
  domains: { domain: string; count: number }[];
  rooms: { area_id: string; name: string; count: number; high: number; medium: number; low: number; inbox: number }[];
  assignment: Assignment[];
  traffic: {
    total: number;
    by_source: Record<string, number>;
    by_intent: Record<string, number>;
    by_day: { day: string; count: number }[];
    clarify: number;
    chat: number;
    empty: number;
    recent: BundleEntry[];
  };
};

export type UiState = {
  tab: Tab;
  locale: Locale;
  dismissed: string[];
  last_apply: ApplyRow[];
  graph: Record<string, { x: number; y: number }>;
};

export type BundleEntry = {
  id: string;
  ts_ms: number;
  source: string;
  language?: string;
  text: string;
  speech: string;
  intents: string[];
  clarify: boolean;
  chat: boolean;
};

export type BundleList = {
  enabled: boolean;
  count: number;
  bytes: number;
  entries: BundleEntry[];
};

export type ApplyRow = {
  entity_id: string;
  before?: string | null;
  after: string;
};

export type Gaps = {
  leftover: Entity[];
  rooms: Area[];
};
