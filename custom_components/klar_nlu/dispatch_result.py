"""Shared intent-step result for dispatch helpers."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class IntentStepResult:
    ok: bool
    speech: str | None = None
    error: str | None = None


def ok(speech: str | None = None) -> IntentStepResult:
    """Mark a step as successful. Empty speech is still success (H-17)."""
    return IntentStepResult(True, speech=speech or None)


def fail(error: str) -> IntentStepResult:
    return IntentStepResult(False, error=error[:256])
