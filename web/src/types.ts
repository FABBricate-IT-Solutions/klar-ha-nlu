import {
  LANGUAGE_TIERS,
  POLICY_EFFECTS,
  REFINE_BANDS as GENERATED_REFINE_BANDS,
  type LanguageTier,
  type Mode,
  type PolicyEffect,
  type RefineBand,
  type RejectReason,
  type UnitSystem,
} from "./generated/contract";

export type { LanguageTier, Mode, PolicyEffect, RefineBand, RejectReason, UnitSystem };
export { LANGUAGE_TIERS, POLICY_EFFECTS };
export const REFINE_BANDS: RefineBand[] = [...GENERATED_REFINE_BANDS];

export type Locale = string;
export type Tab = "home" | "conversations" | "rules" | "house" | "lab" | "settings";
export type HouseView = "graph" | "entities" | "calibrate";
export type RulesView = "routines" | "sentences" | "policies";
export type Theme = "dark" | "light";
export type SettingsView = "llm" | "voice" | "languages" | "engine" | "backup";
export type Confidence = "high" | "medium" | "low";

export type Settings = {
  personality: string;
  mode: Mode;
  languages: string[];
  /** Pack used when `languages` is empty and the request names no language. Empty means English. */
  default_language: string;
  /** Offer untested locales for a new selection. Configured ones stay either way. */
  experimental_languages: boolean;
  support_bundle: boolean;
  support_bundle_raw_text: boolean;
  confirm_risky_actions: boolean;
  semantic_adapters: boolean;
  nlu_rag: boolean;
  /** Wikipedia/Wikidata for out-of-domain chat. Off by default. */
  web_lookup?: boolean;
  /** News briefing RSS fetches. Off by default. */
  news_briefing?: boolean;
  refine_speech?: boolean;
  refine_bands?: RefineBand[];
  calendar_llm?: boolean;
  quiet_ack?: boolean;
  satellite_followup?: boolean;
  allow_llm_tools?: boolean;
  fallback_llm?: boolean;
  /** IANA timezone from Home Assistant. Empty uses the process timezone, then UTC. */
  time_zone?: string;
  extra_prompt?: string;
  unit_system?: UnitSystem;
  custom_voice?: string;
  custom_voice_name?: string;
  custom_voice_seed?: string;
  custom_voice_traits?: VoiceTraits;
  news_feeds?: { pack: string; url: string }[];
};

export function effectiveRefineBands(settings: Settings): RefineBand[] {
  if (Array.isArray(settings.refine_bands)) return settings.refine_bands;
  return settings.refine_speech ? ["status"] : [];
}

export type VoiceTraits = {
  warmth: number;
  humor: number;
  sarcasm: number;
  formality: number;
  verbosity: number;
  energy: number;
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
  | { type: "reject"; reason: RejectReason | string }
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
  /** Present on a news briefing that named a topic; omitted otherwise. */
  news_topic?: string | null;
  confidence: number;
  margin: number;
  candidates: IntentCandidate[];
  evidence: Evidence[];
  trace: ParseTrace;
  retrieval?: Retrieval;
  policy_trace?: PolicyTrace;
  quiet_ack_eligible?: boolean;
  refine_band?: RefineBand | null;
};

export type RetrievalHit = { entity_id: string; name: string; domain: string; area?: string | null };
export type Retrieval = {
  entities?: RetrievalHit[];
  areas?: string[];
  last?: string[];
  custom?: string[];
  tokens?: string[];
};
export type PolicyTraceMatch = { id: string; score: number; origin: string };
export type PolicyTraceLayer = { id: string; hit?: string | null; origin: string };
export type PolicyTraceDiscarded = { id: string; score: number; reason: string };
export type PolicyTrace = {
  matched_rule?: string | null;
  hit?: string | null;
  compiled_risky?: boolean;
  payload?: string | null;
  match?: PolicyTraceMatch | null;
  seed?: PolicyTraceLayer | null;
  house?: PolicyTraceLayer | null;
  band?: string | null;
  discarded?: PolicyTraceDiscarded[];
};

export type PolicyMatch = {
  intent?: string;
  domain?: string;
  area?: string;
  entity_id?: string;
  floor?: string;
  name?: string;
  phrase?: string;
  area_wide?: boolean;
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
export type PolicyBundle = { policies: PolicyRule[]; speech_bank: SpeechBank; match_controls?: MatchControl[] };
export type MatchControl = { id: string; enabled: boolean; precedence?: number };
export type SetDelta = { add?: string[]; remove?: string[] };
export type LanguageOverlay = { sets?: Record<string, SetDelta> };
export type MatchCatalogRow = { id: string; precedence: number; summary_key: string };
export type MatchCatalog = { matches: MatchCatalogRow[]; seeds?: PolicyRule[] };
export type EvaluateOut = {
  outcome: ParseResult;
  compiled_risky: boolean;
  matched_rule?: string | null;
  hit?: string | null;
  speech_variant?: string | null;
  warnings?: string[];
};

export type TrainerContext = {
  language: string;
  layer: string;
  prompt_version: string;
  graph: { areas: Area[]; floors: Floor[]; entities: Entity[] };
  gaps: string[];
  matches: MatchCatalogRow[];
  seeds: PolicyRule[];
  overlays: { policies: PolicyRule[]; match_controls: MatchControl[]; language: LanguageOverlay };
  schema: { effects: string[]; when_fields: string[]; max_rules: number; seed_ids: string[]; match_ids: string[] };
};

export type TrainerIssue = { path: string; message: string };
export type TrainerDryRun = { text: string; decision: string; seed?: string | null; house?: string | null; compiled_risky: boolean };
export type TrainerValidateOut = { ok: boolean; errors: TrainerIssue[]; warnings: TrainerIssue[]; dry_run: TrainerDryRun[] };
export type TrainerProposal = {
  layer?: string;
  language?: string;
  policies?: PolicyRule[];
  match_controls?: MatchControl[];
  language_overlay?: LanguageOverlay;
  utterances?: string[];
};
export type TrainerTurn = { role: "user" | "assistant"; content: string };
export type LlmPublic = {
  configured: boolean;
  base_url?: string;
  model?: string;
  enable_thinking?: boolean;
  provider?: string;
};
export type LlmModels = { models: string[] };
export type RefineOutcome = { type: string; text: string; accepted: boolean };
export type TrainerConsent = {
  call_id: string;
  tool: string;
  summary: string;
  validate: TrainerValidateOut;
};
export type TrainerChatEvent =
  | { type: "delta"; text: string }
  | { type: "done"; text: string }
  | { type: "error"; message: string }
  | { type: "proposal"; value: TrainerProposal }
  | { type: "validate"; value: TrainerValidateOut }
  | { type: "consent"; call_id: string; tool: string; summary: string; validate: TrainerValidateOut }
  | { type: "session"; yolo: boolean; allowed: string[] }
  | { type: "tool_call"; id: string; name: string; arguments: string }
  | {
      type: "tool";
      tool: string;
      text?: string | null;
      intent?: string | null;
      slots?: Record<string, string> | null;
    };

export type ConversationTurn = {
  conversation_id: string;
  ts_ms: number;
  text?: string | null;
  tokens?: string[];
  decision: string;
  speech: string;
  confidence: number;
  briefing: boolean;
  evidence_kinds: string[];
  last_names: string[];
  confirm_prompt?: string | null;
  candidate_id?: string | null;
  preferred_area?: string | null;
  speech_source?: string | null;
  refine_band?: RefineBand | null;
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
  floors?: { floor_id: string; name: string }[];
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
  llm?: LlmWindow;
};

export type LlmWindow = {
  calls: number;
  errors: number;
  accepted: number;
  rejected: number;
  by_kind: Record<string, number>;
  by_day: { day: string; refine: number; assist: number; chat: number }[];
  p50_ms?: number | null;
  p90_ms?: number | null;
  tokens?: { prompt: number; completion: number; total: number; calls_with_usage: number } | null;
};

export type UiState = {
  tab: Tab;
  locale: Locale;
  locale_set?: boolean;
  dismissed: string[];
  last_apply: ApplyRow[];
  graph: Record<string, { x: number; y: number }>;
  wizard_done?: boolean;
  house_view?: HouseView;
  rules_view?: RulesView;
  settings_view?: SettingsView;
  theme?: Theme;
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
