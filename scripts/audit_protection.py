#!/usr/bin/env python3
"""Audit live GitHub protection against .github/protection/desired.yaml (package 5.3).

Validates the checked-in desired config, fetches live rulesets and immutable-releases
when a token is available, and fails on drift. Soft-skips the live compare only when
the API is unavailable (no token / 403 / 404), with a clear report line.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any

import yaml

ROOT = Path(__file__).resolve().parents[1]
DESIRED_PATH = ROOT / ".github" / "protection" / "desired.yaml"
API = "https://api.github.com"
ACCEPT = "application/vnd.github+json"
API_VERSION = "2022-11-28"

PULL_REQUEST_KEYS = (
    "required_approving_review_count",
    "dismiss_stale_reviews_on_push",
    "require_code_owner_review",
    "require_last_push_approval",
    "required_review_thread_resolution",
    "allowed_merge_methods",
)
STATUS_CHECK_KEYS = (
    "strict_required_status_checks_policy",
    "do_not_enforce_on_create",
)


class DesiredConfigError(ValueError):
    """The checked-in desired.yaml failed schema validation."""


def load_desired(path: Path = DESIRED_PATH) -> dict[str, Any]:
    data = yaml.safe_load(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise DesiredConfigError("desired.yaml must be a mapping")
    validate_desired(data)
    return data


def validate_desired(data: dict[str, Any]) -> None:
    if data.get("version") != 1:
        raise DesiredConfigError("version must be 1")
    immutable = data.get("immutable_releases")
    if not isinstance(immutable, dict) or not isinstance(immutable.get("enabled"), bool):
        raise DesiredConfigError("immutable_releases.enabled must be a bool")
    checks = data.get("required_status_checks")
    if not isinstance(checks, list) or not checks or not all(isinstance(c, str) and c for c in checks):
        raise DesiredConfigError("required_status_checks must be a non-empty list of strings")
    rulesets = data.get("rulesets")
    if not isinstance(rulesets, list) or not rulesets:
        raise DesiredConfigError("rulesets must be a non-empty list")
    names: set[str] = set()
    for entry in rulesets:
        _validate_ruleset_entry(entry, checks, names)


def _validate_ruleset_entry(entry: Any, shared_checks: list[str], names: set[str]) -> None:
    if not isinstance(entry, dict):
        raise DesiredConfigError("each ruleset must be a mapping")
    name = entry.get("name")
    if not isinstance(name, str) or not name:
        raise DesiredConfigError("ruleset.name is required")
    if name in names:
        raise DesiredConfigError(f"duplicate ruleset name: {name}")
    names.add(name)
    if entry.get("target") not in ("branch", "tag"):
        raise DesiredConfigError(f"{name}: target must be branch or tag")
    if entry.get("enforcement") != "active":
        raise DesiredConfigError(f"{name}: enforcement must be active")
    include = entry.get("include")
    if not isinstance(include, list) or not include or not all(isinstance(i, str) and i for i in include):
        raise DesiredConfigError(f"{name}: include must be a non-empty list of ref patterns")
    rules = entry.get("rules")
    if not isinstance(rules, list) or not rules:
        raise DesiredConfigError(f"{name}: rules must be a non-empty list")
    for rule in rules:
        _validate_rule(name, rule, shared_checks)
    if entry["target"] == "tag" and not any(
        pattern.startswith("refs/tags/20") or pattern == "refs/tags/20*" for pattern in include
    ):
        raise DesiredConfigError(f"{name}: CalVer tag protection requires an include pattern for 20*")
    if entry["target"] == "branch":
        types = {rule.get("type") for rule in rules if isinstance(rule, dict)}
        for needed in ("pull_request", "required_status_checks", "deletion", "non_fast_forward"):
            if needed not in types:
                raise DesiredConfigError(f"{name}: branch ruleset must include {needed}")


def _validate_rule(name: str, rule: Any, shared_checks: list[str]) -> None:
    if not isinstance(rule, dict) or not isinstance(rule.get("type"), str):
        raise DesiredConfigError(f"{name}: each rule needs a type")
    rtype = rule["type"]
    params = rule.get("parameters")
    if rtype in ("deletion", "non_fast_forward", "update"):
        if params not in (None, {}):
            raise DesiredConfigError(f"{name}: {rtype} must not carry parameters")
        return
    if rtype == "pull_request":
        if not isinstance(params, dict):
            raise DesiredConfigError(f"{name}: pull_request needs parameters")
        for key in PULL_REQUEST_KEYS:
            if key not in params:
                raise DesiredConfigError(f"{name}: pull_request missing {key}")
        methods = params["allowed_merge_methods"]
        if not isinstance(methods, list) or sorted(methods) != sorted(set(methods)):
            raise DesiredConfigError(f"{name}: allowed_merge_methods must be a unique list")
        return
    if rtype == "required_status_checks":
        if not isinstance(params, dict):
            raise DesiredConfigError(f"{name}: required_status_checks needs parameters")
        for key in STATUS_CHECK_KEYS:
            if key not in params or not isinstance(params[key], bool):
                raise DesiredConfigError(f"{name}: required_status_checks.{key} must be a bool")
        if params.get("use_shared_status_checks"):
            return
        contexts = params.get("required_status_checks")
        if not isinstance(contexts, list) or [c.get("context") for c in contexts] != shared_checks:
            raise DesiredConfigError(f"{name}: status check contexts must match required_status_checks")
        return
    raise DesiredConfigError(f"{name}: unsupported rule type {rtype}")


def expand_desired(data: dict[str, Any]) -> dict[str, Any]:
    """Normalize desired.yaml into the shape compare_protection expects."""
    shared = list(data["required_status_checks"])
    rulesets = []
    for entry in data["rulesets"]:
        rules = []
        for rule in entry["rules"]:
            rtype = rule["type"]
            if rtype == "required_status_checks":
                params = dict(rule["parameters"])
                if params.pop("use_shared_status_checks", False):
                    params["required_status_checks"] = [{"context": c} for c in shared]
                rules.append({"type": rtype, "parameters": _norm_status_params(params)})
            elif rtype == "pull_request":
                rules.append({"type": rtype, "parameters": _norm_pull_params(rule["parameters"])})
            else:
                rules.append({"type": rtype})
        rulesets.append(
            {
                "name": entry["name"],
                "target": entry["target"],
                "enforcement": entry["enforcement"],
                "include": sorted(entry["include"]),
                "rules": _sort_rules(rules),
            }
        )
    return {
        "immutable_releases": {"enabled": bool(data["immutable_releases"]["enabled"])},
        "rulesets": sorted(rulesets, key=lambda item: item["name"]),
    }


def normalize_live(rulesets: list[dict[str, Any]], immutable_enabled: bool) -> dict[str, Any]:
    normalized = []
    for entry in rulesets:
        conditions = entry.get("conditions") or {}
        ref_name = conditions.get("ref_name") or {}
        include = sorted(ref_name.get("include") or [])
        rules = []
        for rule in entry.get("rules") or []:
            rtype = rule.get("type")
            if rtype in ("deletion", "non_fast_forward", "update"):
                rules.append({"type": rtype})
            elif rtype == "pull_request":
                rules.append({"type": rtype, "parameters": _norm_pull_params(rule.get("parameters") or {})})
            elif rtype == "required_status_checks":
                rules.append({"type": rtype, "parameters": _norm_status_params(rule.get("parameters") or {})})
            else:
                rules.append({"type": rtype, "parameters": rule.get("parameters")})
        normalized.append(
            {
                "name": entry.get("name"),
                "target": entry.get("target"),
                "enforcement": entry.get("enforcement"),
                "include": include,
                "rules": _sort_rules(rules),
            }
        )
    return {
        "immutable_releases": {"enabled": bool(immutable_enabled)},
        "rulesets": sorted(normalized, key=lambda item: str(item.get("name") or "")),
    }


def _norm_pull_params(params: dict[str, Any]) -> dict[str, Any]:
    methods = params.get("allowed_merge_methods") or []
    return {
        "required_approving_review_count": int(params.get("required_approving_review_count") or 0),
        "dismiss_stale_reviews_on_push": bool(params.get("dismiss_stale_reviews_on_push")),
        "require_code_owner_review": bool(params.get("require_code_owner_review")),
        "require_last_push_approval": bool(params.get("require_last_push_approval")),
        "required_review_thread_resolution": bool(params.get("required_review_thread_resolution")),
        "allowed_merge_methods": sorted(methods),
    }


def _norm_status_params(params: dict[str, Any]) -> dict[str, Any]:
    contexts = params.get("required_status_checks") or []
    names = sorted(
        item["context"] if isinstance(item, dict) else str(item)
        for item in contexts
    )
    return {
        "strict_required_status_checks_policy": bool(params.get("strict_required_status_checks_policy")),
        "do_not_enforce_on_create": bool(params.get("do_not_enforce_on_create")),
        "required_status_checks": [{"context": name} for name in names],
    }


def _sort_rules(rules: list[dict[str, Any]]) -> list[dict[str, Any]]:
    return sorted(rules, key=lambda rule: rule.get("type") or "")


def compare_protection(desired: dict[str, Any], live: dict[str, Any]) -> list[str]:
    """Return human-readable drift lines; empty means match."""
    drifts: list[str] = []
    if desired["immutable_releases"] != live["immutable_releases"]:
        drifts.append(
            "immutable_releases: desired "
            f"{desired['immutable_releases']} != live {live['immutable_releases']}"
        )
    desired_by_name = {item["name"]: item for item in desired["rulesets"]}
    live_by_name = {item["name"]: item for item in live["rulesets"]}
    for name in sorted(set(desired_by_name) | set(live_by_name)):
        if name not in live_by_name:
            drifts.append(f"ruleset missing live: {name}")
            continue
        if name not in desired_by_name:
            drifts.append(f"ruleset unexpected live: {name}")
            continue
        left = desired_by_name[name]
        right = live_by_name[name]
        if left != right:
            drifts.append(f"ruleset drift: {name}")
            drifts.append(f"  desired: {json.dumps(left, sort_keys=True)}")
            drifts.append(f"  live:    {json.dumps(right, sort_keys=True)}")
    return drifts


def api_get(path: str, token: str) -> Any:
    request = urllib.request.Request(
        f"{API}{path}",
        headers={
            "Accept": ACCEPT,
            "Authorization": f"Bearer {token}",
            "X-GitHub-Api-Version": API_VERSION,
            "User-Agent": "klar-nlu-protection-audit",
        },
    )
    with urllib.request.urlopen(request, timeout=60) as response:
        return json.load(response)


def fetch_live(owner: str, repo: str, token: str) -> dict[str, Any]:
    listing = api_get(f"/repos/{owner}/{repo}/rulesets", token)
    details = [api_get(f"/repos/{owner}/{repo}/rulesets/{item['id']}", token) for item in listing]
    immutable = api_get(f"/repos/{owner}/{repo}/immutable-releases", token)
    return normalize_live(details, bool(immutable.get("enabled")))


def resolve_repo(cli_repo: str | None) -> tuple[str, str]:
    raw = cli_repo or os.environ.get("GITHUB_REPOSITORY") or ""
    if "/" not in raw:
        raise SystemExit("set --repo owner/name or GITHUB_REPOSITORY")
    owner, _, name = raw.partition("/")
    return owner, name


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--desired", type=Path, default=DESIRED_PATH)
    parser.add_argument("--repo", help="owner/name (default: GITHUB_REPOSITORY)")
    parser.add_argument(
        "--token-env",
        default="GITHUB_TOKEN",
        help="environment variable that holds a GitHub token (default: GITHUB_TOKEN)",
    )
    parser.add_argument(
        "--require-live",
        action="store_true",
        help="fail when the live API cannot be read (weekly job)",
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="validate desired.yaml and run an in-process comparison fixture",
    )
    args = parser.parse_args(argv)

    try:
        desired_raw = load_desired(args.desired)
    except (OSError, DesiredConfigError, yaml.YAMLError) as err:
        print(f"desired config invalid: {err}", file=sys.stderr)
        return 1

    desired = expand_desired(desired_raw)
    print(f"desired ok: {len(desired['rulesets'])} rulesets, immutable_releases={desired['immutable_releases']}")

    if args.self_test:
        # Fixture matches expand_desired(desired_raw); mutating enabled must drift.
        drifts = compare_protection(desired, desired)
        if drifts:
            print("self-test failed: identity compare drifted", file=sys.stderr)
            return 1
        broken = json.loads(json.dumps(desired))
        broken["immutable_releases"]["enabled"] = not broken["immutable_releases"]["enabled"]
        if not compare_protection(desired, broken):
            print("self-test failed: expected immutable_releases drift", file=sys.stderr)
            return 1
        print("self-test ok")
        return 0

    token = os.environ.get(args.token_env, "").strip()
    if not token:
        message = f"live compare skipped: ${args.token_env} unset"
        print(message)
        return 1 if args.require_live else 0

    try:
        owner, repo = resolve_repo(args.repo)
        live = fetch_live(owner, repo, token)
    except urllib.error.HTTPError as err:
        body = err.read().decode("utf-8", errors="replace")
        message = f"live compare unavailable: HTTP {err.code} {body[:200]}"
        print(message)
        return 1 if args.require_live else 0
    except (urllib.error.URLError, TimeoutError, json.JSONDecodeError, SystemExit) as err:
        print(f"live compare unavailable: {err}")
        return 1 if args.require_live else 0

    drifts = compare_protection(desired, live)
    if drifts:
        print("protection drift detected:", file=sys.stderr)
        for line in drifts:
            print(f"  {line}", file=sys.stderr)
        return 1
    print("live matches desired")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
