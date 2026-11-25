"""Read a compiled pack back from its Rust source.

The hand-written de and en packs exist only as Rust, and a regional pack is built from
its parent's word lists, so the generator reads the parent the way the engine compiles it.
"""

from __future__ import annotations

import re
from pathlib import Path

PACKS = Path(__file__).resolve().parents[2] / "src" / "lang" / "packs"

TOKEN = re.compile(
    r"""\s*(?:
    (?P<comment>//[^\n]*)|
    (?P<str>"(?:[^"\\]|\\.)*")|
    (?P<num>-?\d+(?:\.\d+)?)|
    (?P<ident>[A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)*)|
    (?P<punct>[{}\[\]()&,:;=.])
    )""",
    re.X,
)


def _unescape(raw: str) -> str:
    body = raw[1:-1]
    out: list[str] = []
    i = 0
    while i < len(body):
        char = body[i]
        if char != "\\":
            out.append(char)
            i += 1
            continue
        nxt = body[i + 1]
        if nxt == "u":
            end = body.index("}", i)
            out.append(chr(int(body[i + 3 : end], 16)))
            i = end + 1
            continue
        if nxt == "\n":
            i += 2
            while i < len(body) and body[i] in " \t\n":
                i += 1
            continue
        out.append({"n": "\n", "t": "\t"}.get(nxt, nxt))
        i += 2
    return "".join(out)


def _tokens(src: str) -> list[tuple[str, str]]:
    pos = 0
    out: list[tuple[str, str]] = []
    while pos < len(src):
        match = TOKEN.match(src, pos)
        if not match:
            if not src[pos:].strip():
                break
            raise SyntaxError(f"unreadable Rust at {pos}: {src[pos:pos + 40]!r}")
        pos = match.end()
        for kind in ("str", "num", "ident", "punct"):
            if match.group(kind) is not None:
                out.append((kind, match.group(kind)))
                break
    return out


class _Reader:
    def __init__(self, toks: list[tuple[str, str]]):
        self.toks = toks
        self.i = 0

    def peek(self) -> tuple[str | None, str | None]:
        return self.toks[self.i] if self.i < len(self.toks) else (None, None)

    def eat(self, want: str | None = None) -> str:
        kind, val = self.toks[self.i]
        if want is not None and val != want:
            raise SyntaxError(f"expected {want}, got {val}")
        self.i += 1
        return val

    def value(self):
        kind, val = self.peek()
        if val == "&":
            self.eat()
            return self.value()
        if kind == "str":
            return _unescape(self.eat())
        if kind == "num":
            self.eat()
            return float(val) if "." in val else int(val)
        if val == "[":
            return self.seq("[", "]")
        if val == "(":
            return tuple(self.seq("(", ")"))
        if kind == "ident":
            self.eat()
            follow = self.peek()[1]
            if follow == "{":
                return self.struct()
            if follow == "(":
                inner = self.seq("(", ")")
                return inner[0] if val == "Some" else {"call": val, "args": inner}
            return None if val == "None" else {"ident": val}
        raise SyntaxError(f"unexpected {val}")

    def seq(self, open_: str, close: str) -> list:
        self.eat(open_)
        items = []
        while self.peek()[1] != close:
            items.append(self.value())
            if self.peek()[1] == ",":
                self.eat()
        self.eat(close)
        return items

    def struct(self) -> dict:
        self.eat("{")
        out = {}
        while self.peek()[1] != "}":
            if self.peek()[1] == ".":
                # `..MoodWords::EMPTY`: the fields not written out are empty.
                self.eat(".")
                self.eat(".")
                self.eat()
                continue
            key = self.eat()
            self.eat(":")
            out[key] = self.value()
            if self.peek()[1] == ",":
                self.eat()
        self.eat("}")
        return out


def _const(path: Path, name: str):
    src = path.read_text(encoding="utf-8")
    match = re.search(r"const\s+" + name + r"\s*:\s*[^=]+=\s*", src)
    if not match:
        raise KeyError(f"{name} not in {path}")
    return _Reader(_tokens(src[match.end() :])).value()


def read_pack(mod: str) -> dict:
    """Word lists of `src/lang/packs/<mod>` keyed like the pack struct, plus `verbs` as (word, kind)."""
    base = PACKS / mod
    pack = _const(base / "pack.rs", "PACK")
    pack["verbs"] = [(word, kind["ident"].split("::")[-1]) for word, kind in _const(base / "verbs.rs", "VERBS")]
    if "ident" in pack.get("answers", {}):
        # Hand-written packs keep their answers in a file of their own.
        pack["answers"] = _const(base / "answers.rs", "ANSWERS")
    mood = _const(base / "mood.rs", "MOOD")
    pack["mood"] = mood if "ident" not in mood else {}
    style = pack["maps"].get("number_style")
    if isinstance(style, dict):
        pack["maps"]["number_style"] = style["ident"].split("::")[-1]
    return pack
