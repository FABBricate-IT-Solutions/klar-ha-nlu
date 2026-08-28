export type Locale = string;
export type Tab = "home" | "conversations" | "rules" | "house" | "lab" | "settings";
export type HouseView = "graph" | "entities" | "calibrate";
export type Confidence = "high" | "medium" | "low";

export type Settings = {
  personality: string;
  mode: "full" | "context_only";
  languages: string[];
  support_bundle: boolean;
  support_bundle_raw_text: boolean;
  confirm_risky_actions: boolean;
  semantic_adapters: boolean;
  nlu_rag: boolean;
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
  floor_id?: string | null;
};

export type Floor = {
  floor_id: string;
  name: string;
  aliases: string[];
  level?: number | null;
};

export type Slot = { name: string; value: string };
export type Intent = { name: string; slots: Slot[] };
export type Evidence = { kind: string; source: string; value: string; score: number; exact: boolean };
export type PlanStep = { index: number; intent: Intent; confidence: number; evidence: Evidence[] };
export type IntentPlan = { confidence: number; margin: number; evidence: Evidence[]; steps: PlanStep[] };
export type IntentCandidate = {
  id: string;
  plan: IntentPlan;
  score: number;
  margin: number;
  policy: string;
  precedence: number;
  evidence: Evidence[];
};
export type ParseTrace = {
  stages: { stage: string; duration_us: number; detail: string }[];
  discarded: { candidate_id: string; policy: string; score: number; reason: string }[];
  tokens?: string[];
  normalized?: string;
};

export type ParseDecision =
  | { type: "execute" }
  | { type: "clarify"; prompt: string; options: string[] }
  | { type: "confirm"; prompt: string; candidate_id: string }
  | { type: "reject"; reason: string }
  | { type: "chat" }
  | { type: "error"; code: string; message: string };

export type ParseResult = {
  schema_version: string;
  text: string;
  speech: string;
  conversation_id: string;
  decision: ParseDecision;
  plan?: IntentPlan;
  selected_candidate_id?: string;
  briefing: boolean;
  confidence: number;
  margin: number;
  candidates: IntentCandidate[];
  evidence: Evidence[];
  trace: ParseTrace;
  retrieval?: Retrieval;
  policy_trace?: PolicyTrace;
};

export type RetrievalHit = { entity_id: string; name: string; domain: string; area?: string | null };
export type Retrieval = {
  entities?: RetrievalHit[];
  areas?: string[];
  last?: string[];
  custom?: string[];
  tokens?: string[];
};
export type PolicyTrace = {
  matched_rule?: string | null;
  hit?: string | null;
  compiled_risky?: boolean;
  payload?: string | null;
};

export type PolicyEffect = "confirm" | "block" | "allow" | "prefer_entity" | "prefer_area" | "reply" | "script" | "template" | "llm";
export type PolicyMatch = {
  intent?: string;
  domain?: string;
  area?: string;
  entity_id?: string;
  floor?: string;
  name?: string;
  phrase?: string;
};
export type PolicyRule = {
  id: string;
  enabled: boolean;
  label: string;
  when: PolicyMatch;
  effect: PolicyEffect;
  prefer?: string;
  payload?: string;
};
export type SpeechVariant = { language: string; personality: string; text: string };
export type SpeechBankEntry = { rule_id: string; variants: SpeechVariant[] };
export type SpeechBank = { entries: SpeechBankEntry[] };
export type PolicyBundle = { policies: PolicyRule[]; speech_bank: SpeechBank };
export type EvaluateOut = {
  outcome: ParseResult;
  compiled_risky: boolean;
  matched_rule?: string | null;
  hit?: string | null;
  speech_variant?: string | null;
};

export type ConversationTurn = {
  conversation_id: string;
  ts_ms: number;
  text?: string | null;
  decision: string;
  speech: string;
  confidence: number;
  briefing: boolean;
  evidence_kinds: string[];
  last_names: string[];
  confirm_prompt?: string | null;
  candidate_id?: string | null;
  preferred_area?: string | null;
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
