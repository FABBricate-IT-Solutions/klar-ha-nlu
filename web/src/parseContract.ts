import type { ParseDecision, ParseResult } from "./types";

const MAX_STEPS = 32;
const MAX_CANDIDATES = 64;
const MAX_EVIDENCE = 128;
const MAX_ITEM_EVIDENCE = 16;
const MAX_OPTIONS = 32;
const MAX_STAGES = 16;
const MAX_DISCARDED = 64;
const MAX_DETAIL = 256;
const MAX_TOKENS = 256;
const MAX_TOKEN = 64;

type JsonObject = Record<string, unknown>;

export function parseV2Response(value: unknown): ParseResult {
  const payload = object(value, "response");
  keys(
    payload,
    [
      "schema_version",
      "text",
      "conversation_id",
      "decision",
      "speech",
      "confidence",
      "margin",
      "selected_candidate_id",
      "candidates",
      "plan",
      "evidence",
      "trace",
      "briefing",
      "retrieval",
      "policy_trace",
    ],
    ["selected_candidate_id", "plan", "retrieval", "policy_trace"],
  );
  if (payload.schema_version !== "2.0") throw new Error("Unsupported parse schema");
  string(payload.text, "text", 4096, true);
  string(payload.conversation_id, "conversation_id", 128);
  string(payload.speech, "speech", 4096, true);
  score(payload.confidence, "confidence");
  score(payload.margin, "margin");
  if (payload.selected_candidate_id !== undefined) string(payload.selected_candidate_id, "selected_candidate_id", 128);
  if (typeof payload.briefing !== "boolean") throw new Error("briefing must be boolean");
  const decision = validateDecision(payload.decision);
  if (decision === "execute") {
    validatePlan(payload.plan, "plan", true);
  } else if (payload.plan !== undefined && payload.plan !== null) {
    throw new Error("Non-execute response contains an executable plan");
  }
  const candidates = list<JsonObject>(payload.candidates, "candidates", MAX_CANDIDATES, validateCandidate);
  if (decision === "execute") {
    if (typeof payload.selected_candidate_id !== "string") throw new Error("Execute response has no selected candidate");
    const selected = candidates.find((candidate) => candidate.id === payload.selected_candidate_id);
    if (selected === undefined || JSON.stringify(selected.plan) !== JSON.stringify(payload.plan)) {
      throw new Error("Selected candidate does not match execute plan");
    }
  } else if (payload.selected_candidate_id !== undefined || candidates.length !== 0) {
    throw new Error("Non-execute response exposes candidates or a selected candidate");
  }
  list(payload.evidence, "evidence", MAX_EVIDENCE, validateEvidence);
  validateTrace(payload.trace);
  if (payload.policy_trace !== undefined && payload.policy_trace !== null) {
    validatePolicyTrace(payload.policy_trace);
  }
  if (payload.retrieval !== undefined && payload.retrieval !== null) {
    if (decision !== "chat" && decision !== "reject") {
      throw new Error("retrieval only allowed on chat or reject");
    }
  }
  return payload as ParseResult;
}

function validateDecision(value: unknown): string {
  const decision = object(value, "decision");
  const kind = decision.type;
  if (typeof kind !== "string" || !["execute", "clarify", "confirm", "reject", "chat", "error"].includes(kind)) {
    throw new Error("Unknown decision");
  }
  const decisionType = kind as ParseDecision["type"];
  switch (decisionType) {
    case "execute":
    case "chat":
      keys(decision, ["type"]);
      break;
    case "clarify":
      keys(decision, ["type", "prompt", "options"]);
      string(decision.prompt, "decision.prompt", MAX_DETAIL);
      list(decision.options, "decision.options", MAX_OPTIONS, (item, path) => string(item, path, 128));
      break;
    case "confirm":
      keys(decision, ["type", "prompt", "candidate_id"]);
      string(decision.prompt, "decision.prompt", MAX_DETAIL);
      string(decision.candidate_id, "decision.candidate_id", 128);
      break;
    case "reject":
      keys(decision, ["type", "reason"]);
      string(decision.reason, "decision.reason", 64);
      break;
    case "error":
      keys(decision, ["type", "code", "message"]);
      string(decision.code, "decision.code", 64);
      string(decision.message, "decision.message", MAX_DETAIL);
      break;
    default: {
      const exhaustive: never = decisionType;
      throw new Error(`Unhandled decision ${exhaustive}`);
    }
  }
  return decisionType;
}

function validateCandidate(value: unknown, path: string): void {
  const candidate = object(value, path);
  keys(candidate, ["id", "plan", "score", "margin", "policy", "precedence", "evidence"]);
  string(candidate.id, `${path}.id`, 128);
  validatePlan(candidate.plan, `${path}.plan`, false);
  score(candidate.score, `${path}.score`);
  score(candidate.margin, `${path}.margin`);
  string(candidate.policy, `${path}.policy`, 128);
  integer(candidate.precedence, `${path}.precedence`, 65535);
  list(candidate.evidence, `${path}.evidence`, MAX_ITEM_EVIDENCE, validateEvidence);
}

function validatePlan(value: unknown, path: string, requireSteps: boolean): void {
  const plan = object(value, path);
  keys(plan, ["confidence", "margin", "evidence", "steps"]);
  score(plan.confidence, `${path}.confidence`);
  score(plan.margin, `${path}.margin`);
  list(plan.evidence, `${path}.evidence`, MAX_ITEM_EVIDENCE, validateEvidence);
  const steps = list(plan.steps, `${path}.steps`, MAX_STEPS, validateStep);
  if (requireSteps && steps.length === 0) throw new Error("Execute plan is empty");
}

function validateStep(value: unknown, path: string): void {
  const step = object(value, path);
  keys(step, ["index", "intent", "confidence", "evidence"]);
  integer(step.index, `${path}.index`, MAX_STEPS - 1);
  score(step.confidence, `${path}.confidence`);
  list(step.evidence, `${path}.evidence`, MAX_ITEM_EVIDENCE, validateEvidence);
  const intent = object(step.intent, `${path}.intent`);
  keys(intent, ["name", "slots"]);
  string(intent.name, `${path}.intent.name`, 128);
  list(intent.slots, `${path}.intent.slots`, 32, (item, slotPath) => {
    const slot = object(item, slotPath);
    keys(slot, ["name", "value"]);
    string(slot.name, `${slotPath}.name`, 64);
    string(slot.value, `${slotPath}.value`, 512);
  });
}

function validateEvidence(value: unknown, path: string): void {
  const evidence = object(value, path);
  keys(evidence, ["kind", "source", "value", "score", "exact"]);
  string(evidence.kind, `${path}.kind`, 64);
  string(evidence.source, `${path}.source`, 128);
  string(evidence.value, `${path}.value`, 512);
  score(evidence.score, `${path}.score`);
  if (typeof evidence.exact !== "boolean") throw new Error(`${path}.exact must be boolean`);
}

function validatePolicyTrace(value: unknown): void {
  const trace = object(value, "policy_trace");
  keys(
    trace,
    ["matched_rule", "hit", "compiled_risky", "payload", "match", "seed", "house", "band", "discarded"],
    ["matched_rule", "hit", "compiled_risky", "payload", "match", "seed", "house", "band", "discarded"],
  );
  if (trace.matched_rule !== undefined && trace.matched_rule !== null) {
    string(trace.matched_rule, "policy_trace.matched_rule", 64);
  }
  if (trace.hit !== undefined && trace.hit !== null) {
    string(trace.hit, "policy_trace.hit", 32);
  }
  if (trace.payload !== undefined && trace.payload !== null) {
    string(trace.payload, "policy_trace.payload", 500, true);
  }
  if (trace.compiled_risky !== undefined && typeof trace.compiled_risky !== "boolean") {
    throw new Error("policy_trace.compiled_risky must be boolean");
  }
  if (trace.match !== undefined && trace.match !== null) {
    validatePolicyMatch(trace.match, "policy_trace.match");
  }
  if (trace.seed !== undefined && trace.seed !== null) {
    validatePolicyLayer(trace.seed, "policy_trace.seed");
  }
  if (trace.house !== undefined && trace.house !== null) {
    validatePolicyLayer(trace.house, "policy_trace.house");
  }
  if (trace.band !== undefined && trace.band !== null) {
    string(trace.band, "policy_trace.band", 32);
  }
  if (trace.discarded !== undefined) {
    list(trace.discarded, "policy_trace.discarded", MAX_DISCARDED, (item, path) => {
      const discarded = object(item, path);
      keys(discarded, ["id", "score", "reason"]);
      string(discarded.id, `${path}.id`, 128);
      score(discarded.score, `${path}.score`);
      string(discarded.reason, `${path}.reason`, MAX_DETAIL);
    });
  }
}

function validatePolicyMatch(value: unknown, path: string): void {
  const node = object(value, path);
  keys(node, ["id", "score", "origin"]);
  string(node.id, `${path}.id`, 128);
  score(node.score, `${path}.score`);
  string(node.origin, `${path}.origin`, 32);
}

function validatePolicyLayer(value: unknown, path: string): void {
  const layer = object(value, path);
  keys(layer, ["id", "hit", "origin"], ["hit"]);
  string(layer.id, `${path}.id`, 128);
  string(layer.origin, `${path}.origin`, 32);
  if (layer.hit !== undefined && layer.hit !== null) {
    string(layer.hit, `${path}.hit`, 32);
  }
}

function validateTrace(value: unknown): void {
  const trace = object(value, "trace");
  keys(trace, ["stages", "discarded", "tokens", "normalized"], ["tokens", "normalized"]);
  list(trace.stages, "trace.stages", MAX_STAGES, (item, path) => {
    const stage = object(item, path);
    keys(stage, ["stage", "duration_us", "detail"]);
    string(stage.stage, `${path}.stage`, 64);
    integer(stage.duration_us, `${path}.duration_us`, Number.MAX_SAFE_INTEGER);
    string(stage.detail, `${path}.detail`, MAX_DETAIL, true);
  });
  list(trace.discarded, "trace.discarded", MAX_DISCARDED, (item, path) => {
    const discarded = object(item, path);
    keys(discarded, ["candidate_id", "policy", "score", "reason"]);
    string(discarded.candidate_id, `${path}.candidate_id`, 128);
    string(discarded.policy, `${path}.policy`, 128);
    score(discarded.score, `${path}.score`);
    string(discarded.reason, `${path}.reason`, MAX_DETAIL);
  });
  if (trace.tokens !== undefined) {
    list(trace.tokens, "trace.tokens", MAX_TOKENS, (item, path) => string(item, path, MAX_TOKEN));
  }
  if (trace.normalized !== undefined) {
    string(trace.normalized, "trace.normalized", 4096, true);
  }
}

function object(value: unknown, path: string): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) throw new Error(`${path} must be an object`);
  return value as JsonObject;
}

function keys(value: JsonObject, allowed: string[], optional: string[] = []): void {
  if (allowed.some((key) => !optional.includes(key) && !(key in value))) throw new Error("Response is missing fields");
}

function list<T = unknown>(
  value: unknown,
  path: string,
  maximum: number,
  validate: (item: unknown, path: string) => void,
): T[] {
  if (!Array.isArray(value) || value.length > maximum) throw new Error(`${path} must be a capped array`);
  value.forEach((item, index) => validate(item, `${path}[${index}]`));
  return value as T[];
}

function string(value: unknown, path: string, maximum: number, allowEmpty = false): void {
  if (typeof value !== "string" || value.length > maximum || (!allowEmpty && value.length === 0)) {
    throw new Error(`${path} must be a capped string`);
  }
}

function score(value: unknown, path: string): void {
  if (typeof value !== "number" || !Number.isFinite(value) || value < 0 || value > 1) throw new Error(`${path} must be a score`);
}

function integer(value: unknown, path: string, maximum: number): void {
  if (!Number.isInteger(value) || (value as number) < 0 || (value as number) > maximum) throw new Error(`${path} must be an integer`);
}
