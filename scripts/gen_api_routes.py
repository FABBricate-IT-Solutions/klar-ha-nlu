#!/usr/bin/env python3
"""Generate the HTTP route index in docs/en/api.md and docs/api.md from the router.

Source of truth: `.route(...)` registrations under `src/io/` (excluding `*_tests.rs`),
plus the capability table in `src/io/auth.rs`. Re-run after adding a route; CI /
`scripts/check_generated.sh` fail when the docs lag.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
IO = ROOT / "src" / "io"
AUTH = IO / "auth.rs"
BEGIN = "<!-- BEGIN GENERATED ROUTES -->"
END = "<!-- END GENERATED ROUTES -->"

# One-line purpose per path (English). German twin uses DE_BLURB.
BLURB: dict[str, str] = {
    "/": "React operator UI (or redirect into `web/dist`).",
    "/api/v2/health": "Public build identity, home age, overlay, LLM, Wyoming, ops counters (no token).",
    "/api/v2/setup/token": "Supervisor/loopback claim of the auto-minted write token (fresh App install).",
    "/api/v2/parse": "V2 parse (`schema_version: \"2.0\"`).",
    "/api/settings": "Engine settings store (GET/POST full, PATCH partial).",
    "/api/custom": "Custom phrase list.",
    "/api/entities": "Home graph listing; POST tags / aliases / preferred / area.",
    "/api/gaps": "Calibration leftovers and rooms.",
    "/api/dashboard": "Operator dashboard payload for the React UI.",
    "/api/ui": "Persistent UI chrome state under the overlay.",
    "/api/assignment/apply": "Apply room-assignment suggestions.",
    "/api/assignment/undo": "Undo the last assignment batch.",
    "/api/bundle": "Support-bundle status; DELETE clears it.",
    "/api/bundle/clear": "Clear the support-bundle protocol (POST alias).",
    "/api/bundle/entries": "List or delete recorded turns.",
    "/api/bundle/dataset": "Download marked-wrong cases as nlu_expect YAML with redacted home snippets.",
    "/api/bundle/protocol": "Download the redacted JSONL protocol.",
    "/api/v2/conversations": "Conversation journal (last 200 turns / 24 h).",
    "/api/v2/conversations/live": "SSE stream of journal turns (snapshot, then each new turn).",
    "/api/v2/conversations/{id}": "Turns for one conversation id.",
    "/api/v2/conversations/mark-wrong": "Mark a journal turn with the intended intent/target; export an nlu_expect case.",
    "/api/v2/wrong-cases": "List operator-marked regression cases.",
    "/api/v2/wrong-cases/dataset": "Download marked-wrong cases as nlu_expect YAML.",
    "/api/v2/wrong-cases/dry-run": "Dry-run saved per-home regression cases against the live engine.",
    "/api/v2/wrong-cases/update-dialog": "After an engine update: how many saved cases changed, and which sentences.",
    "/api/v2/last-turn": "Newest journal turn, or null.",
    "/api/v2/home": "Live home-graph push from the Home Assistant integration.",
    "/api/v2/languages": "Locales that can be picked (tier + offered).",
    "/api/v2/intents": "Known intent name list.",
    "/api/lang/overlay": "User language overlay + custom sentences.",
    "/api/lang/preview": "Parse with unsaved overlay; does not install.",
    "/api/lang/explain": "Preview plus stages / evidence / matched custom.",
    "/api/lang/rollback": "Restore a previous overlay history row.",
    "/api/v2/policies": "Rules tab policy bundle.",
    "/api/v2/policies/catalog": "Built-in match catalog and safety seeds.",
    "/api/v2/policies/evaluate": "Dry-run parse against optional policies.",
    "/api/v2/policies/preview-turns": "Count how many recent journal turns a draft rule would match.",
    "/api/v2/policies/history": "List prior house-rule bundle snapshots.",
    "/api/v2/policies/rollback": "Restore a prior house-rule bundle from history.",
    "/api/v2/policies/trainer-context": "Trainer context stub for the Rules LLM.",
    "/api/v2/policies/propose/validate": "Validate a trainer proposal without applying.",
    "/api/v2/policies/trainer/chat": "Trainer chat SSE (write token).",
    "/api/v2/policies/trainer/consent": "Approve or deny a pending trainer write tool.",
    "/api/v2/settings/history": "List prior engine settings snapshots.",
    "/api/v2/settings/rollback": "Restore prior engine settings from history.",
    "/api/v2/session/undo-priors": "Stamp prior brightness/position/temperature/volume onto the last execute so undo can restore them.",
    "/api/v2/speech/render": "Post-execute speech from an HA state snapshot.",
    "/api/v2/speech/asr_boost": "Ranked Whisper bias terms for external STT.",
    "/api/v2/settings/backup": "Download allowlisted settings tar.gz.",
    "/api/v2/settings/restore": "Restore from a settings backup archive.",
    "/api/v2/llm/endpoint": "OpenAI-compatible upstream config (never returns the key).",
    "/api/v2/llm/voice": "Personality preview (flavor + refine prompt).",
    "/api/v2/llm/custom-voice": "Generate a custom voice block via the endpoint LLM.",
    "/api/v2/llm/models": "List models from a caller-named upstream (write).",
    "/api/v2/llm/chat": "Raw chat / SSE against the engine LLM.",
    "/api/v2/llm/refine": "Engine-owned refine of finished NLU speech.",
    "/api/v2/llm/assist": "Engine-owned Yarn/chat/RAG/calendar/news assist.",
}

DE_BLURB: dict[str, str] = {
    "/": "React-Operator-UI (oder Redirect nach `web/dist`).",
    "/api/v2/health": "Öffentliche Build-Identität, Home-Alter, Overlay, LLM, Wyoming, Ops-Zähler (ohne Token).",
    "/api/v2/setup/token": "Supervisor/Loopback holt das automatisch erzeugte Schreib-Token (frische App-Installation).",
    "/api/v2/parse": "V2-Parse (`schema_version: \"2.0\"`).",
    "/api/settings": "Engine-Settings-Store (GET/POST vollständig, PATCH partiell).",
    "/api/custom": "Liste der Custom-Phrasen.",
    "/api/entities": "Home-Graph; POST Tags / Aliase / preferred / area.",
    "/api/gaps": "Kalibrierungs-Rest und Räume.",
    "/api/dashboard": "Dashboard-Payload für die React-UI.",
    "/api/ui": "Persistenter UI-Zustand im Overlay.",
    "/api/assignment/apply": "Raum-Zuordnungsvorschläge anwenden.",
    "/api/assignment/undo": "Letzten Assignment-Batch rückgängig.",
    "/api/bundle": "Support-Bundle-Status; DELETE leert ihn.",
    "/api/bundle/clear": "Support-Bundle-Protokoll leeren (POST-Alias).",
    "/api/bundle/entries": "Aufgezeichnete Turns listen oder löschen.",
    "/api/bundle/dataset": "Als falsch markierte Fälle als nlu_expect-YAML mit redigiertem Home-Ausschnitt laden.",
    "/api/bundle/protocol": "Redigiertes JSONL-Protokoll laden.",
    "/api/v2/conversations": "Gesprächsjournal (letzte 200 Turns / 24 h).",
    "/api/v2/conversations/live": "SSE-Stream der Journal-Turns (Snapshot, dann jeder neue Turn).",
    "/api/v2/conversations/{id}": "Turns einer Conversation-ID.",
    "/api/v2/conversations/mark-wrong": "Journal-Turn mit gemeintem Intent/Ziel markieren; nlu_expect-Fall exportieren.",
    "/api/v2/wrong-cases": "Vom Operator markierte Regressionsfälle listen.",
    "/api/v2/wrong-cases/dataset": "Als falsch markierte Fälle als nlu_expect-YAML laden.",
    "/api/v2/wrong-cases/dry-run": "Gespeicherte Haus-Regressionsfälle trocken gegen die laufende Engine laufen lassen.",
    "/api/v2/wrong-cases/update-dialog": "Nach Engine-Update: wie viele gespeicherte Fälle sich ändern, und welche Sätze.",
    "/api/v2/last-turn": "Neuester Journal-Turn, oder null.",
    "/api/v2/home": "Live-Home-Graph von der Home-Assistant-Integration.",
    "/api/v2/languages": "Wählbare Locales (Tier + offered).",
    "/api/v2/intents": "Liste der bekannten Intent-Namen.",
    "/api/lang/overlay": "Sprach-Overlay + Custom-Sätze.",
    "/api/lang/preview": "Parse mit ungespeichertem Overlay; installiert nicht.",
    "/api/lang/explain": "Preview plus Stages / Evidence / matched custom.",
    "/api/lang/rollback": "Frühere Overlay-History-Zeile wiederherstellen.",
    "/api/v2/policies": "Policy-Bundle der Rules-Registerkarte.",
    "/api/v2/policies/catalog": "Eingebauter Match-Katalog und Safety-Seeds.",
    "/api/v2/policies/evaluate": "Dry-Run-Parse gegen optionale Policies.",
    "/api/v2/policies/preview-turns": "Zählt, wie viele jüngste Journal-Turns eine Entwurfsregel treffen würde.",
    "/api/v2/policies/history": "Frühere Hausregel-Snapshots listen.",
    "/api/v2/policies/rollback": "Früheres Hausregel-Bundle aus der Historie wiederherstellen.",
    "/api/v2/policies/trainer-context": "Trainer-Kontext für das Rules-LLM.",
    "/api/v2/policies/propose/validate": "Trainer-Vorschlag prüfen, ohne anzuwenden.",
    "/api/v2/policies/trainer/chat": "Trainer-Chat SSE (Write-Token).",
    "/api/v2/policies/trainer/consent": "Ausstehende Trainer-Write-Tools freigeben oder ablehnen.",
    "/api/v2/settings/history": "Frühere Engine-Settings-Snapshots listen.",
    "/api/v2/settings/rollback": "Frühere Engine-Settings aus der Historie wiederherstellen.",
    "/api/v2/session/undo-priors": "Vorherige Helligkeit/Position/Temperatur/Lautstärke auf den letzten Execute stempeln, damit Undo sie wiederherstellen kann.",
    "/api/v2/speech/render": "Sprache nach Execute aus HA-Zustandsschnappschuss.",
    "/api/v2/speech/asr_boost": "Whisper-Bias-Liste für externes STT.",
    "/api/v2/settings/backup": "Allowlistetes Settings-tar.gz laden.",
    "/api/v2/settings/restore": "Settings aus Backup-Archiv wiederherstellen.",
    "/api/v2/llm/endpoint": "OpenAI-kompatibler Upstream (Key nie im GET).",
    "/api/v2/llm/voice": "Persönlichkeits-Vorschau (Flavor + Refine-Prompt).",
    "/api/v2/llm/custom-voice": "Custom-Voice-Block über das Endpoint-LLM erzeugen.",
    "/api/v2/llm/models": "Modelle eines genannten Upstreams listen (Write).",
    "/api/v2/llm/chat": "Rohes Chat/SSE gegen das Engine-LLM.",
    "/api/v2/llm/refine": "Engine-eigenes Refine fertiger NLU-Sprache.",
    "/api/v2/llm/assist": "Engine-eigenes Yarn/Chat/RAG/Kalender/News-Assist.",
}

ROUTE_RE = re.compile(
    r'\.route\(\s*"([^"]+)"\s*,\s*((?:[a-z_]+(?:\([^()]*\))?\s*\.?\s*)+)',
    re.I,
)
METHOD_RE = re.compile(r"\b(get|post|put|delete|patch)\(", re.I)
CAP_RE = re.compile(
    r'&Method::(GET|POST|PUT|DELETE|PATCH),\s*"([^"]+)",\s*Capability::(\w+)',
)


def router_routes() -> dict[str, list[str]]:
    """path -> sorted uppercase methods from axum `.route` registrations."""
    found: dict[str, set[str]] = {}
    for path in sorted(IO.glob("*.rs")):
        if "test" in path.name:
            continue
        text = path.read_text(encoding="utf-8")
        cut = text.find("#[cfg(test)]")
        body = text if cut < 0 else text[:cut]
        for match in ROUTE_RE.finditer(body):
            route = match.group(1)
            methods = {m.upper() for m in METHOD_RE.findall(match.group(2))}
            found.setdefault(route, set()).update(methods)
    if not found:
        raise SystemExit("no .route() registrations under src/io/")
    return {path: sorted(methods) for path, methods in sorted(found.items())}


def capabilities() -> dict[tuple[str, str], str]:
    """(METHOD, path) -> capability name from auth.rs ROUTES."""
    text = AUTH.read_text(encoding="utf-8")
    out: dict[tuple[str, str], str] = {}
    for method, path, cap in CAP_RE.findall(text):
        out[(method, path)] = cap
    return out


def render_table(lang: str) -> str:
    routes = router_routes()
    caps = capabilities()
    blurbs = DE_BLURB if lang == "de" else BLURB
    auth_header = "Auth" if lang == "en" else "Auth"
    purpose = "Purpose" if lang == "en" else "Zweck"
    public = "public" if lang == "en" else "öffentlich"
    lines = [
        BEGIN,
        "",
        f"| Method | Path | {auth_header} | {purpose} |",
        "|--------|------|------|---------|",
    ]
    for path, methods in routes.items():
        blurb = blurbs.get(path, "")
        if not blurb:
            raise SystemExit(f"missing {lang} blurb for route {path}")
        for method in methods:
            cap = caps.get((method, path))
            auth = public if cap is None else cap
            lines.append(f"| `{method}` | `{path}` | {auth} | {blurb} |")
    lines.extend(["", END])
    return "\n".join(lines) + "\n"


def replace_block(path: Path, block: str) -> None:
    text = path.read_text(encoding="utf-8")
    if BEGIN not in text or END not in text:
        raise SystemExit(f"{path}: missing {BEGIN} / {END} markers")
    start = text.index(BEGIN)
    stop = text.index(END) + len(END)
    # Keep a single trailing newline after the block.
    after = text[stop:]
    if after.startswith("\n"):
        after = after[1:]
    path.write_text(text[:start] + block + after, encoding="utf-8")


def write_docs() -> None:
    replace_block(ROOT / "docs" / "en" / "api.md", render_table("en"))
    replace_block(ROOT / "docs" / "api.md", render_table("de"))


def main(argv: list[str]) -> int:
    if argv == ["--check"]:
        before_en = (ROOT / "docs" / "en" / "api.md").read_text(encoding="utf-8")
        before_de = (ROOT / "docs" / "api.md").read_text(encoding="utf-8")
        write_docs()
        after_en = (ROOT / "docs" / "en" / "api.md").read_text(encoding="utf-8")
        after_de = (ROOT / "docs" / "api.md").read_text(encoding="utf-8")
        if before_en != after_en or before_de != after_de:
            # restore so --check is non-destructive when used from a test that
            # already copied files; here we rewrite then compare — write back.
            (ROOT / "docs" / "en" / "api.md").write_text(before_en, encoding="utf-8")
            (ROOT / "docs" / "api.md").write_text(before_de, encoding="utf-8")
            print("api route docs are stale; run: python3 scripts/gen_api_routes.py", file=sys.stderr)
            return 1
        return 0
    if argv:
        raise SystemExit("usage: gen_api_routes.py [--check]")
    write_docs()
    print("updated docs/en/api.md and docs/api.md")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
