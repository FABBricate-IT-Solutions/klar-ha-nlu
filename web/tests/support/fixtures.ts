import type { LangOverlay, LanguagePack } from "../../src/api";
import type {
  BundleList,
  Dashboard,
  Evidence,
  IntentPlan,
  LlmPublic,
  MatchCatalog,
  ParseDecision,
  ParseResult,
  PolicyBundle,
  Settings,
  UiState,
} from "../../src/types";

// What a lived-in house has stored. None of these equal the UI's built-in defaults, so a test can
// tell "the UI sent back what it loaded" from "the UI sent its own empty state".

export const storedSettings: Settings = {
  personality: "butler",
  mode: "full",
  languages: ["de"],
  support_bundle: true,
  support_bundle_raw_text: false,
  confirm_risky_actions: true,
  semantic_adapters: false,
  nlu_rag: false,
  refine_speech: true,
  refine_bands: ["status", "command"],
  calendar_llm: false,
  quiet_ack: true,
  satellite_followup: false,
  allow_llm_tools: false,
  fallback_llm: true,
  extra_prompt: "Sprich kurz.",
  unit_system: "metric",
  custom_voice: "",
  custom_voice_name: "",
  custom_voice_seed: "",
  custom_voice_traits: { warmth: 7, humor: 4, sarcasm: 2, formality: 5, verbosity: 4, energy: 5 },
  news_feeds: [{ pack: "de", url: "https://example.invalid/feed.xml" }],
};

export const storedUi: UiState = {
  tab: "home",
  locale: "en",
  locale_set: true,
  dismissed: ["light.bedroom_globe"],
  last_apply: [],
  graph: { "light.living_room": { x: 40, y: 60 } },
  wizard_done: true,
  house_view: "calibrate",
  rules_view: "routines",
  settings_view: "llm",
  theme: "dark",
};

export const storedPolicies: PolicyBundle = {
  policies: [
    { id: "rule-ac", enabled: true, label: "No AC at night", when: { domain: "climate" }, effect: "confirm" },
    { id: "rule-garage", enabled: true, label: "Garage asks first", when: { domain: "cover" }, effect: "confirm" },
  ],
  speech_bank: { entries: [] },
  match_controls: [],
};

export const storedOverlay: LangOverlay = {
  custom: [{ phrase: "gute nacht", intent: "HassTurnOn", slots: { entity_id: "script.good_night" } }],
  language: { sets: {} },
  history: [],
};

export const storedLlm: LlmPublic = {
  configured: true,
  base_url: "https://llm.example.invalid/v1",
  model: "house-model",
  enable_thinking: false,
  provider: "custom",
};

export const storedDashboard: Dashboard = {
  counts: { all: 3, assist: 3, rooms: 2, leftover: 1, high: 2, medium: 1, low: 0, bundle: 0 },
  coverage: { all: 3, assist: 3, high: 2, leftover: 1 },
  domains: [{ domain: "light", count: 2 }, { domain: "cover", count: 1 }],
  rooms: [
    { area_id: "living_room", name: "Living room", count: 2, high: 2, medium: 0, low: 0, inbox: 0 },
    { area_id: "garage", name: "Garage", count: 1, high: 0, medium: 1, low: 0, inbox: 1 },
  ],
  floors: [],
  assignment: [],
  traffic: { total: 0, by_source: {}, by_intent: {}, by_day: [], clarify: 0, chat: 0, empty: 0, recent: [] },
};

export const languagePacks: LanguagePack[] = [
  { code: "de", native_name: "Deutsch", script: "Latn", variants: [] },
  { code: "en", native_name: "English", script: "Latn", variants: [] },
];

export const matchCatalog: MatchCatalog = {
  matches: [{ id: "all_lights", precedence: 5, summary_key: "match.all_lights" }],
  seeds: [],
};

export const emptyBundle: BundleList = { enabled: false, count: 0, bytes: 0, entries: [] };

type ParseOverrides = Partial<Omit<ParseResult, "decision" | "plan">> & {
  decision?: ParseDecision;
  /** Intent names of the plan; each becomes one step. Only used for an `execute` decision. */
  intents?: string[];
  slots?: Record<string, string>;
};

/**
 * A parse answer in the engine's v2 wire shape (same fields as a live `/api/v2/parse` response), so
 * it passes `parseV2Response`. Defaults to an executed "turn on the hallway lamp".
 */
export function parseResult(overrides: ParseOverrides = {}): ParseResult {
  const { decision = { type: "execute" }, intents = ["HassTurnOn"], slots = { entity_id: "light.hallway_lamp" }, ...rest } = overrides;
  const evidence: Evidence[] = [{ kind: "action", source: "lexicon_exact", value: "on", score: 1, exact: true }];
  const base: ParseResult = {
    schema_version: "2.0",
    text: "turn on the hallway lamp",
    conversation_id: "conv-1",
    decision,
    speech: "The hallway lamp is on.",
    confidence: 0.88,
    margin: 0.2,
    briefing: false,
    candidates: [],
    evidence,
    trace: { stages: [{ stage: "normalize", duration_us: 40, detail: "" }], discarded: [] },
  };
  if (decision.type === "execute") {
    const plan: IntentPlan = {
      confidence: 0.88,
      margin: 0.2,
      evidence,
      steps: intents.map((name, index) => ({
        index,
        intent: { name, slots: Object.entries(slots).map(([slot, value]) => ({ name: slot, value })) },
        confidence: 0.88,
        evidence,
      })),
    };
    base.plan = plan;
    base.selected_candidate_id = "plan-000";
    base.candidates = [{ id: "plan-000", plan, score: 0.88, margin: 0.2, policy: "grounded_entities", precedence: 12, evidence }];
  }
  return { ...base, ...rest };
}
