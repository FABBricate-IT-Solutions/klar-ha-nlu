"""Strict runtime validation for the Klar V2 parse contract."""

from __future__ import annotations

import math
from typing import Any

MAX_STEPS = 32
MAX_CANDIDATES = 64
MAX_EVIDENCE = 128
MAX_EVIDENCE_PER_ITEM = 16
MAX_CLARIFY_OPTIONS = 32
MAX_TRACE_STAGES = 16
MAX_TRACE_DISCARDED = 64
MAX_DETAIL_CHARS = 256
MAX_EXECUTE_ERROR_CHARS = 256
_DECISIONS = {"execute", "clarify", "confirm", "reject", "chat", "error"}
_EXECUTE_OUTCOMES = {"success", "partial", "error"}
_STEP_STATUSES = {"success", "error"}


def validate_v2_payload(value: Any) -> dict[str, Any]:
    payload = _mapping(value, "response")
    _keys(
        payload,
        {
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
            "quiet_ack_eligible",
        },
        "response",
        optional={"selected_candidate_id", "plan", "retrieval", "policy_trace", "quiet_ack_eligible"},
    )
    if payload.get("schema_version") != "2.0":
        raise ValueError("unsupported Klar schema_version")
    _capped_string(payload.get("text"), "text", 4096)
    _string(payload.get("conversation_id"), "conversation_id", 128)
    _capped_string(payload.get("speech"), "speech", 4096)
    _score(payload.get("confidence"), "confidence")
    _score(payload.get("margin"), "margin")
    selected = payload.get("selected_candidate_id")
    if selected is not None:
        _string(selected, "selected_candidate_id", 128)
    if not isinstance(payload.get("briefing"), bool):
        raise ValueError("briefing must be boolean")
    decision = _decision(payload.get("decision"))
    plan = payload.get("plan")
    if decision == "execute":
        _plan(plan, "plan", require_steps=True)
    elif plan is not None:
        raise ValueError("non-execute response must not contain plan")
    candidates = _list(payload.get("candidates"), "candidates", MAX_CANDIDATES, _candidate)
    if decision == "execute":
        selected_id = _string(selected, "selected_candidate_id", 128)
        selected_candidate = next((candidate for candidate in candidates if candidate.get("id") == selected_id), None)
        if selected_candidate is None or selected_candidate.get("plan") != plan:
            raise ValueError("selected candidate does not match execute plan")
    elif selected is not None or candidates:
        raise ValueError("non-execute response must not expose candidates or a selected candidate")
    _list(payload.get("evidence"), "evidence", MAX_EVIDENCE, _evidence)
    _trace(payload.get("trace"))
    retrieval = payload.get("retrieval")
    if retrieval is not None:
        if decision not in {"chat", "reject"}:
            raise ValueError("retrieval only allowed on chat or reject")
        _retrieval(retrieval)
    if payload.get("policy_trace") is not None:
        _policy_trace(payload.get("policy_trace"))
    eligible = payload.get("quiet_ack_eligible")
    if eligible is not None and not isinstance(eligible, bool):
        raise ValueError("quiet_ack_eligible must be boolean")
    return payload


def validate_execute_result(value: Any) -> dict[str, Any]:
    result = _mapping(value, "execute_result")
    _keys(result, {"outcome", "speech", "steps"}, "execute_result")
    if result.get("outcome") not in _EXECUTE_OUTCOMES:
        raise ValueError("unknown execute outcome")
    _capped_string(result.get("speech"), "execute_result.speech", 4096)
    steps = _list(result.get("steps"), "execute_result.steps", MAX_STEPS, _execute_step)
    statuses = [step.get("status") for step in steps]
    if result["outcome"] == "success" and (not steps or any(status != "success" for status in statuses)):
        raise ValueError("success requires every step to succeed")
    if result["outcome"] == "partial" and not (any(status == "success" for status in statuses) and any(status == "error" for status in statuses)):
        raise ValueError("partial requires mixed step results")
    if result["outcome"] == "error" and steps and any(status == "success" for status in statuses):
        raise ValueError("error must not include successful steps")
    return result


def _execute_step(value: Any, path: str) -> None:
    step = _mapping(value, path)
    _keys(step, {"index", "intent", "status", "speech", "error"}, path, optional={"speech", "error"})
    index = step.get("index")
    if not isinstance(index, int) or isinstance(index, bool) or index < 0:
        raise ValueError(f"{path}.index is invalid")
    _string(step.get("intent"), f"{path}.intent", 128)
    if step.get("status") not in _STEP_STATUSES:
        raise ValueError(f"{path}.status is invalid")
    speech = step.get("speech")
    if speech is not None:
        _capped_string(speech, f"{path}.speech", 4096)
    error = step.get("error")
    if error is not None:
        _capped_string(error, f"{path}.error", MAX_EXECUTE_ERROR_CHARS)
    if step.get("status") == "success" and error:
        raise ValueError(f"{path} success must not include error")
    if step.get("status") == "error" and not error:
        raise ValueError(f"{path} error must include a message")


def executable_intents(payload: dict[str, Any]) -> list[dict[str, Any]]:
    decision = payload.get("decision")
    if not isinstance(decision, dict) or decision.get("type") != "execute":
        return []
    plan = payload.get("plan")
    if not isinstance(plan, dict):
        return []
    return [
        step["intent"]
        for step in plan.get("steps", [])
        if isinstance(step, dict) and isinstance(step.get("intent"), dict)
    ]


def _decision(value: Any) -> str:
    decision = _mapping(value, "decision")
    kind = decision.get("type")
    if kind not in _DECISIONS:
        raise ValueError("unknown decision type")
    allowed = {
        "execute": {"type"},
        "clarify": {"type", "prompt", "options"},
        "confirm": {"type", "prompt", "candidate_id"},
        "reject": {"type", "reason"},
        "chat": {"type"},
        "error": {"type", "code", "message"},
    }[kind]
    _keys(decision, allowed, "decision")
    if kind == "clarify":
        _string(decision.get("prompt"), "decision.prompt", MAX_DETAIL_CHARS)
        _list(decision.get("options"), "decision.options", MAX_CLARIFY_OPTIONS, lambda item, path: _string(item, path, 128))
    elif kind == "confirm":
        _string(decision.get("prompt"), "decision.prompt", MAX_DETAIL_CHARS)
        _string(decision.get("candidate_id"), "decision.candidate_id", 128)
    elif kind == "reject":
        _string(decision.get("reason"), "decision.reason", 64)
    elif kind == "error":
        _string(decision.get("code"), "decision.code", 64)
        _string(decision.get("message"), "decision.message", MAX_DETAIL_CHARS)
    return kind


def _candidate(value: Any, path: str) -> None:
    candidate = _mapping(value, path)
    _keys(candidate, {"id", "plan", "score", "margin", "policy", "precedence", "evidence"}, path)
    _string(candidate.get("id"), f"{path}.id", 128)
    _plan(candidate.get("plan"), f"{path}.plan", require_steps=False)
    _score(candidate.get("score"), f"{path}.score")
    _score(candidate.get("margin"), f"{path}.margin")
    _string(candidate.get("policy"), f"{path}.policy", 128)
    precedence = candidate.get("precedence")
    if not isinstance(precedence, int) or isinstance(precedence, bool) or not 0 <= precedence <= 65535:
        raise ValueError(f"{path}.precedence is invalid")
    _list(candidate.get("evidence"), f"{path}.evidence", MAX_EVIDENCE_PER_ITEM, _evidence)


def _plan(value: Any, path: str, *, require_steps: bool) -> None:
    plan = _mapping(value, path)
    _keys(plan, {"confidence", "margin", "evidence", "steps"}, path)
    _score(plan.get("confidence"), f"{path}.confidence")
    _score(plan.get("margin"), f"{path}.margin")
    _list(plan.get("evidence"), f"{path}.evidence", MAX_EVIDENCE_PER_ITEM, _evidence)
    steps = _list(plan.get("steps"), f"{path}.steps", MAX_STEPS, _step)
    if require_steps and not steps:
        raise ValueError("execute plan must contain steps")


def _step(value: Any, path: str) -> None:
    step = _mapping(value, path)
    _keys(step, {"index", "intent", "confidence", "evidence"}, path)
    index = step.get("index")
    if not isinstance(index, int) or isinstance(index, bool) or index < 0:
        raise ValueError(f"{path}.index is invalid")
    _score(step.get("confidence"), f"{path}.confidence")
    _list(step.get("evidence"), f"{path}.evidence", MAX_EVIDENCE_PER_ITEM, _evidence)
    intent = _mapping(step.get("intent"), f"{path}.intent")
    _keys(intent, {"name", "slots"}, f"{path}.intent")
    _string(intent.get("name"), f"{path}.intent.name", 128)
    _list(intent.get("slots"), f"{path}.intent.slots", 32, _slot)


def _slot(value: Any, path: str) -> None:
    slot = _mapping(value, path)
    _keys(slot, {"name", "value"}, path)
    _string(slot.get("name"), f"{path}.name", 64)
    _string(slot.get("value"), f"{path}.value", 512)


def _evidence(value: Any, path: str) -> None:
    evidence = _mapping(value, path)
    _keys(evidence, {"kind", "source", "value", "score", "exact"}, path)
    _string(evidence.get("kind"), f"{path}.kind", 64)
    _string(evidence.get("source"), f"{path}.source", 128)
    _string(evidence.get("value"), f"{path}.value", 512)
    _score(evidence.get("score"), f"{path}.score")
    if not isinstance(evidence.get("exact"), bool):
        raise ValueError(f"{path}.exact must be boolean")


def _retrieval(value: Any) -> None:
    pack = _mapping(value, "retrieval")
    _keys(
        pack,
        {"entities", "areas", "last", "custom", "tokens"},
        "retrieval",
        optional={"entities", "areas", "last", "custom", "tokens"},
    )
    entities = pack.get("entities")
    if entities is not None:
        _list(entities, "retrieval.entities", 8, _retrieval_hit)
    for key in ("areas", "last", "custom"):
        items = pack.get(key)
        if items is not None:
            _list(items, f"retrieval.{key}", 8, lambda item, path: _string(item, path, 128))
    tokens = pack.get("tokens")
    if tokens is not None:
        _list(tokens, "retrieval.tokens", 32, lambda item, path: _string(item, path, 64))


def _retrieval_hit(value: Any, path: str) -> None:
    hit = _mapping(value, path)
    _keys(hit, {"entity_id", "name", "domain", "area"}, path, optional={"area"})
    _string(hit.get("entity_id"), f"{path}.entity_id", 128)
    _string(hit.get("name"), f"{path}.name", 128)
    _string(hit.get("domain"), f"{path}.domain", 32)
    if hit.get("area") is not None:
        _string(hit.get("area"), f"{path}.area", 128)


def _policy_trace(value: Any) -> None:
    trace = _mapping(value, "policy_trace")
    _keys(
        trace,
        {"matched_rule", "hit", "compiled_risky", "payload", "match", "seed", "house", "band", "discarded"},
        "policy_trace",
        optional={"matched_rule", "hit", "compiled_risky", "payload", "match", "seed", "house", "band", "discarded"},
    )
    if trace.get("matched_rule") is not None:
        _string(trace.get("matched_rule"), "policy_trace.matched_rule", 64)
    if trace.get("hit") is not None:
        _string(trace.get("hit"), "policy_trace.hit", 32)
    if trace.get("payload") is not None:
        _string(trace.get("payload"), "policy_trace.payload", 500)
    if "compiled_risky" in trace and not isinstance(trace.get("compiled_risky"), bool):
        raise ValueError("policy_trace.compiled_risky must be boolean")
    if trace.get("match") is not None:
        node = _mapping(trace.get("match"), "policy_trace.match")
        _keys(node, {"id", "score", "origin"}, "policy_trace.match")
        _string(node.get("id"), "policy_trace.match.id", 128)
        _score(node.get("score"), "policy_trace.match.score")
        _string(node.get("origin"), "policy_trace.match.origin", 32)
    for key in ("seed", "house"):
        layer = trace.get(key)
        if layer is None:
            continue
        item = _mapping(layer, f"policy_trace.{key}")
        _keys(item, {"id", "hit", "origin"}, f"policy_trace.{key}", optional={"hit"})
        _string(item.get("id"), f"policy_trace.{key}.id", 128)
        _string(item.get("origin"), f"policy_trace.{key}.origin", 32)
        if item.get("hit") is not None:
            _string(item.get("hit"), f"policy_trace.{key}.hit", 32)
    if trace.get("band") is not None:
        _string(trace.get("band"), "policy_trace.band", 32)
    if trace.get("discarded") is not None:
        _list(trace.get("discarded"), "policy_trace.discarded", MAX_TRACE_DISCARDED, _policy_discarded)


def _policy_discarded(value: Any, path: str) -> None:
    item = _mapping(value, path)
    _keys(item, {"id", "score", "reason"}, path)
    _string(item.get("id"), f"{path}.id", 128)
    _score(item.get("score"), f"{path}.score")
    _string(item.get("reason"), f"{path}.reason", MAX_DETAIL_CHARS)


def _trace(value: Any) -> None:
    trace = _mapping(value, "trace")
    _keys(trace, {"stages", "discarded", "tokens", "normalized"}, "trace", optional={"tokens", "normalized"})
    _list(trace.get("stages"), "trace.stages", MAX_TRACE_STAGES, _stage)
    _list(trace.get("discarded"), "trace.discarded", MAX_TRACE_DISCARDED, _discarded)
    tokens = trace.get("tokens")
    if tokens is not None:
        _list(tokens, "trace.tokens", MAX_TRACE_STAGES * 16, lambda item, path: _string(item, path, 64))
    normalized = trace.get("normalized")
    if normalized is not None:
        _capped_string(normalized, "trace.normalized", MAX_DETAIL_CHARS)


def _stage(value: Any, path: str) -> None:
    stage = _mapping(value, path)
    _keys(stage, {"stage", "duration_us", "detail"}, path)
    _string(stage.get("stage"), f"{path}.stage", 64)
    duration = stage.get("duration_us")
    if not isinstance(duration, int) or isinstance(duration, bool) or duration < 0:
        raise ValueError(f"{path}.duration_us is invalid")
    _string(stage.get("detail"), f"{path}.detail", MAX_DETAIL_CHARS)


def _discarded(value: Any, path: str) -> None:
    item = _mapping(value, path)
    _keys(item, {"candidate_id", "policy", "score", "reason"}, path)
    _string(item.get("candidate_id"), f"{path}.candidate_id", 128)
    _string(item.get("policy"), f"{path}.policy", 128)
    _score(item.get("score"), f"{path}.score")
    _string(item.get("reason"), f"{path}.reason", MAX_DETAIL_CHARS)


def _mapping(value: Any, path: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise ValueError(f"{path} must be an object")
    return value


def _keys(value: dict[str, Any], allowed: set[str], path: str, *, optional: set[str] | None = None) -> None:
    if unknown := set(value) - allowed:
        raise ValueError(f"{path} has unknown fields: {sorted(unknown)}")
    if missing := allowed - (optional or set()) - set(value):
        raise ValueError(f"{path} is missing fields: {sorted(missing)}")


def _list(value: Any, path: str, maximum: int, validator: Any) -> list[Any]:
    if not isinstance(value, list) or len(value) > maximum:
        raise ValueError(f"{path} must be a capped array")
    for index, item in enumerate(value):
        validator(item, f"{path}[{index}]")
    return value


def _string(value: Any, path: str, maximum: int) -> str:
    if not isinstance(value, str) or not value or len(value) > maximum:
        raise ValueError(f"{path} must be a non-empty capped string")
    return value


def _capped_string(value: Any, path: str, maximum: int) -> str:
    if not isinstance(value, str) or len(value) > maximum:
        raise ValueError(f"{path} must be a capped string")
    return value


def _score(value: Any, path: str) -> float:
    if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value) or not 0 <= value <= 1:
        raise ValueError(f"{path} must be a finite score")
    return float(value)
