"""Package 6.11 / Q-13: ADR 0003 boundary, module responsibility and the non-core freeze."""

from __future__ import annotations

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

from guardrails import boundary, focus, responsibility  # noqa: E402
from guardrails.config import GuardrailError, load  # noqa: E402

REGEX_MODULE = 'import re\n\nPATTERN = re.compile(r"turn on (\\w+)")\n'


def write(root: Path, rel: str, text: str) -> None:
    path = root / rel
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def boundary_config(*allow: dict[str, Any]) -> dict[str, Any]:
    return {"boundary": {"root": "custom_components/klar_nlu", "allow": list(allow)}}


def allow(file: str, sites: int, kind: str = "output") -> dict[str, Any]:
    return {"file": file, "sites": sites, "kind": kind, "reason": "test"}


class BoundaryTests(unittest.TestCase):
    def check(self, files: dict[str, str], *entries: dict[str, Any]) -> list[str]:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for name, text in files.items():
                write(root, f"custom_components/klar_nlu/{name}", text)
            return boundary.check(boundary_config(*entries), root)

    def test_new_utterance_regex_fails_and_names_the_adr_and_the_fix(self) -> None:
        problems = self.check({"intent_guess.py": REGEX_MODULE})
        self.assertEqual(len(problems), 1)
        self.assertIn("custom_components/klar_nlu/intent_guess.py:3", problems[0])
        self.assertIn("ADR 0003", problems[0])
        self.assertIn("Rust engine", problems[0])
        self.assertIn("[[boundary.allow]]", problems[0])

    def test_aliases_and_from_imports_are_seen(self) -> None:
        for source in (
            'import re as rx\nrx.search("a", "b")\n',
            'from re import findall as grab\ngrab("a", "b")\n',
            'import regex\nregex.sub("a", "b", "c")\n',
        ):
            with self.subTest(source=source):
                self.assertEqual(len(boundary.regex_sites(source)), 1)

    def test_importing_re_without_matching_and_string_methods_are_fine(self) -> None:
        source = 'import re\n\nWORDS = "a b".split()\nre.escape("x")\nre.Pattern\n'
        self.assertEqual(boundary.regex_sites(source), [])

    def test_allowlisted_file_passes_at_its_ceiling(self) -> None:
        self.assertEqual(self.check({"clock.py": REGEX_MODULE}, allow("clock.py", 1)), [])

    def test_a_second_site_in_an_allowlisted_file_fails(self) -> None:
        text = REGEX_MODULE + 'OTHER = re.compile("x")\n'
        problems = self.check({"clock.py": text}, allow("clock.py", 1))
        self.assertEqual(len(problems), 1)
        self.assertIn("2 regex sites", problems[0])
        self.assertIn("do not raise the allowance", problems[0])

    def test_a_ceiling_above_the_remaining_sites_must_be_lowered(self) -> None:
        problems = self.check({"clock.py": "x = 1\n"}, allow("clock.py", 1))
        self.assertEqual(len(problems), 1)
        self.assertIn("Lower it", problems[0])

    def test_an_entry_for_a_missing_file_fails(self) -> None:
        problems = self.check({"other.py": "x = 1\n"}, allow("gone.py", 1))
        self.assertEqual(len(problems), 1)
        self.assertIn("gone.py", problems[0])

    def test_entries_need_a_known_kind_and_a_positive_ceiling(self) -> None:
        with self.assertRaises(GuardrailError):
            self.check({}, allow("a.py", 1, kind="utterance"))
        with self.assertRaises(GuardrailError):
            self.check({}, allow("a.py", 0))

    def test_repository_is_clean_and_the_one_known_debt_is_named(self) -> None:
        config = load()
        self.assertEqual(boundary.check(config), [])
        debt = [entry["file"] for entry in config["boundary"]["allow"] if entry["kind"] == "debt"]
        self.assertEqual(debt, ["reminders.py"])


def module_config(*modules: dict[str, Any], review: int = 500, ceiling: int = 1000) -> dict[str, Any]:
    return {"responsibility": {"review_lines": review, "ceiling_lines": ceiling, "module": list(modules)}}


def module(path: str, kind: str = "logic", owner: str = "engine") -> dict[str, Any]:
    return {"path": path, "owner": owner, "kind": kind, "job": "reads the thing"}


def lines(count: int) -> str:
    return "".join(f"let x{i} = {i};\n" for i in range(count))


class ResponsibilityTests(unittest.TestCase):
    def check(self, files: dict[str, str], *entries: dict[str, Any]) -> list[str]:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for name, text in files.items():
                write(root, name, text)
            return responsibility.check(module_config(*entries), root)

    def test_files_just_under_and_just_over_the_old_limit_are_not_a_cliff(self) -> None:
        for count in (480, 499, 500):
            with self.subTest(lines=count):
                self.assertEqual(self.check({"src/a.rs": lines(count)}), [])

    def test_a_long_module_without_an_entry_asks_for_a_job_not_a_split(self) -> None:
        problems = self.check({"src/a.rs": lines(501)})
        self.assertEqual(len(problems), 1)
        self.assertIn("src/a.rs has 501 lines", problems[0])
        self.assertIn("one job", problems[0])
        self.assertIn("Do not split the file just to get under a number", problems[0])

    def test_an_entry_covers_the_module_up_to_the_ceiling(self) -> None:
        self.assertEqual(self.check({"src/a.rs": lines(1000)}, module("src/a.rs")), [])

    def test_a_logic_module_past_the_ceiling_is_told_to_split_by_responsibility(self) -> None:
        problems = self.check({"src/a.rs": lines(1001)}, module("src/a.rs"))
        self.assertEqual(len(problems), 1)
        self.assertIn("reads the thing", problems[0])
        self.assertIn("Split along the separate responsibilities", problems[0])

    def test_a_table_has_no_ceiling_but_still_needs_its_entry(self) -> None:
        self.assertEqual(self.check({"scripts/t.py": lines(2000)}, module("scripts/t.py", kind="table")), [])
        self.assertEqual(len(self.check({"scripts/t.py": lines(2000)})), 1)

    def test_generated_files_and_tests_are_owned_by_their_generator_and_their_cases(self) -> None:
        files = {
            "src/lang/g.rs": "// @generated by scripts/lang_packs. DO NOT EDIT.\n" + lines(900),
            "src/io/web_tests.rs": lines(900),
            "src/speech/tests.rs": lines(900),
            "scripts/tests/helper.py": lines(900),
            "custom_components/klar_nlu/test_x.py": lines(900),
        }
        self.assertEqual(self.check(files), [])

    def test_an_entry_for_a_file_that_is_gone_fails_but_a_shrunk_file_does_not(self) -> None:
        self.assertEqual(len(self.check({"src/a.rs": lines(10)}, module("src/gone.rs"))), 1)
        self.assertEqual(self.check({"src/a.rs": lines(10)}, module("src/a.rs")), [])

    def test_entries_need_a_known_owner_and_kind(self) -> None:
        with self.assertRaises(GuardrailError):
            self.check({}, module("src/a.rs", owner="nobody"))
        with self.assertRaises(GuardrailError):
            self.check({}, module("src/a.rs", kind="misc"))

    def test_repository_is_clean_and_has_no_line_limit_left(self) -> None:
        self.assertEqual(responsibility.check(load()), [])
        gate = (ROOT / "scripts" / "check_lang_packs.py").read_text(encoding="utf-8")
        self.assertNotIn("MAX_LINES", gate)
        self.assertNotIn("over 500 lines", gate)
        self.assertNotIn("check_size_gate", gate)


CONFIG: dict[str, Any] = {
    "focus": {
        "ack_trailer": "Frozen-area",
        "ack_reasons": ["security", "regression"],
        "ignored": ["docs/*", "CHANGELOG.md"],
        "max_share_percent": 5,
        "settings": "src/types/settings.rs",
        "area": [
            {"name": "news", "frozen": True, "paths": ["custom_components/klar_nlu/news.py", "tests/ha/test_news.py"],
             "default_off": ["news_briefing"]},
            {"name": "llm", "frozen": False, "paths": ["src/llm/*"]},
        ],
    }
}


def commit(subject: str, *files: str, body: str = "") -> focus.Commit:
    return focus.Commit("a" * 40, subject, body, files)


class FocusCommitTests(unittest.TestCase):
    def problems(self, *commits: focus.Commit) -> list[str]:
        return focus.check_commits(list(commits), CONFIG)

    def test_a_drive_by_news_fix_needs_a_reason(self) -> None:
        problems = self.problems(commit("fix: read the story that was named", "custom_components/klar_nlu/news.py", "tests/ha/test_news.py"))
        self.assertEqual(len(problems), 1)
        self.assertIn("frozen", problems[0])
        self.assertIn("Frozen-area: <reason>", problems[0])

    def test_the_trailer_with_a_listed_reason_lets_it_through(self) -> None:
        body = "Why.\n\nFrozen-area: security\n"
        self.assertEqual(self.problems(commit("fix(news): drop a leaked token", "custom_components/klar_nlu/news.py", body=body)), [])

    def test_an_unlisted_reason_does_not_count(self) -> None:
        body = "Frozen-area: boredom\n"
        self.assertEqual(len(self.problems(commit("fix: x", "custom_components/klar_nlu/news.py", body=body))), 1)

    def test_features_in_a_frozen_area_need_it_too(self) -> None:
        self.assertEqual(len(self.problems(commit("feat: more feeds", "custom_components/klar_nlu/news.py"))), 1)

    def test_core_fixes_other_types_and_llm_only_fixes_need_no_trailer(self) -> None:
        self.assertEqual(self.problems(commit("fix: parse floors", "src/parse/floors.rs")), [])
        self.assertEqual(self.problems(commit("chore: tidy", "custom_components/klar_nlu/news.py")), [])
        self.assertEqual(self.problems(commit("fix: refine prompt", "src/llm/refine.rs")), [])
        self.assertEqual(self.problems(commit("Keep the news parser tidy.", "custom_components/klar_nlu/news.py")), [])

    def test_docs_and_a_core_majority_do_not_make_a_frozen_fix_out_of_a_core_fix(self) -> None:
        mixed = commit("fix: floors", "src/parse/a.rs", "src/parse/b.rs", "custom_components/klar_nlu/news.py")
        self.assertEqual(self.problems(mixed), [])
        docs = commit("fix: floors", "src/parse/a.rs", "docs/news.md", "CHANGELOG.md")
        self.assertEqual(focus.classify(docs, CONFIG), (False, False))

    def test_share_counts_news_and_llm_fixes_against_all_fixes(self) -> None:
        commits = [commit("fix: a", "src/parse/a.rs")] * 18 + [
            commit("fix: n", "custom_components/klar_nlu/news.py"),
            commit("fix: l", "src/llm/refine.rs"),
            commit("feat: not a fix", "src/llm/refine.rs"),
        ]
        self.assertEqual(focus.share(commits, CONFIG), (2, 20))
        percent, text = focus.share_report(commits, CONFIG)
        self.assertEqual(percent, 10.0)
        self.assertIn("2 of 20", text)
        self.assertEqual(focus.share_report([], CONFIG)[0], 0.0)


class FocusStaticTests(unittest.TestCase):
    def setup_root(self, root: Path, *, settings: str, owners: str) -> None:
        write(root, "src/types/settings.rs", settings)
        write(root, ".github/CODEOWNERS", owners)
        write(root, "custom_components/klar_nlu/news.py", "x = 1\n")
        write(root, "tests/ha/test_news.py", "x = 1\n")
        write(root, "src/llm/refine.rs", "\n")

    def owners(self) -> str:
        return "/custom_components/klar_nlu/news.py @o\n/tests/ha/test_news.py @o\n"

    def test_passes_when_frozen_paths_exist_are_owned_and_default_off(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            self.setup_root(Path(tmp), settings="    news_briefing: false,\n", owners=self.owners())
            self.assertEqual(focus.check_static(CONFIG, Path(tmp)), [])

    def test_fails_when_the_default_is_turned_on(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            self.setup_root(Path(tmp), settings="    news_briefing: true,\n", owners=self.owners())
            problems = focus.check_static(CONFIG, Path(tmp))
            self.assertEqual(len(problems), 1)
            self.assertIn("news_briefing: false", problems[0])

    def test_fails_without_a_codeowners_line_or_with_a_dead_path(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.setup_root(root, settings="    news_briefing: false,\n", owners="/tests/ha/test_news.py @o\n")
            (root / "tests/ha/test_news.py").unlink()
            problems = "\n".join(focus.check_static(CONFIG, root))
            self.assertIn("matches no file", problems)
            self.assertIn("not in .github/CODEOWNERS", problems)

    def test_repository_freeze_is_in_place(self) -> None:
        config = load()
        self.assertEqual(focus.check_static(config), [])
        names = {area.name for area in focus.areas(config) if area.frozen}
        self.assertEqual(names, {"news-briefing", "wikipedia-answers", "personality-few-shots", "custom-voice-interview"})
        defaults = {field for area in focus.areas(config) for field in area.default_off}
        self.assertEqual(defaults, {"news_briefing", "web_lookup", "refine_speech"})


class GitHistoryTests(unittest.TestCase):
    def git(self, root: Path, *args: str) -> None:
        subprocess.run(
            ["git", "-c", "user.name=t", "-c", "user.email=t@example.com", "-c", "commit.gpgsign=false", *args],
            cwd=root, check=True, capture_output=True,
        )

    def test_range_check_reads_subject_body_and_files_from_git(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.git(root, "init", "-q", "-b", "staging")
            write(root, "README.md", "x\n")
            self.git(root, "add", "-A")
            self.git(root, "commit", "-q", "-m", "chore: start")
            self.git(root, "checkout", "-q", "-b", "topic")
            write(root, "custom_components/klar_nlu/news.py", "x = 1\n")
            self.git(root, "add", "-A")
            self.git(root, "commit", "-q", "-m", "fix: read the news story")
            write(root, "custom_components/klar_nlu/news.py", "x = 2\n")
            self.git(root, "add", "-A")
            self.git(root, "commit", "-q", "-m", "fix: stop a leak", "-m", "Frozen-area: security")
            commits = focus.git_commits(["staging..HEAD"], root)
            self.assertEqual([c.subject for c in commits], ["fix: stop a leak", "fix: read the news story"])
            self.assertEqual(commits[0].files, ("custom_components/klar_nlu/news.py",))
            problems = focus.check_commits(commits, CONFIG)
            self.assertEqual(len(problems), 1)
            self.assertIn("fix: read the news story", problems[0])


class CommandLineTests(unittest.TestCase):
    def run_cli(self, *args: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(ROOT / "scripts" / "check_guardrails.py"), *args],
            capture_output=True, text=True, check=False,
        )

    def test_static_checks_pass_on_the_repository(self) -> None:
        done = self.run_cli()
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertIn("guardrails ok: boundary, responsibility, focus", done.stdout)

    def test_unknown_check_is_a_usage_error(self) -> None:
        self.assertEqual(self.run_cli("lines").returncode, 2)


if __name__ == "__main__":
    unittest.main()
