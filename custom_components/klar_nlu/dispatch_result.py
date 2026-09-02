"""Shared intent-step result for dispatch helpers."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class IntentStepResult:
    ok: bool
    speech: str | None = None
    error: str | None = None


def ok(speech: str | None) -> IntentStepResult:
    if speech:
        return IntentStepResult(True, speech=speech)
    return IntentStepResult(False, error="empty_speech")


def fail(error: str) -> IntentStepResult:
    return IntentStepResult(False, error=error[:256])
