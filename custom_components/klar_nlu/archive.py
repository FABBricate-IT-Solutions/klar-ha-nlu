"""Checksum and member helpers for the Klar engine archive. No Home Assistant import."""

from __future__ import annotations

import hashlib
import shutil
import tarfile
from io import BytesIO
from pathlib import Path


def is_klar_binary_name(name: str) -> bool:
    path = Path(name)
    if ".." in path.parts:
        return False
    base = path.name
    return base == "klar" or base.startswith("klar-linux-")


def pick_klar_member(names: list[str]) -> str | None:
    candidates = [name for name in names if is_klar_binary_name(name)]
    for name in candidates:
        if Path(name).name == "klar":
            return name
    return candidates[0] if candidates else None


def require_sha256(digest: object, blob: bytes) -> None:
    if not isinstance(digest, str) or not digest.startswith("sha256:"):
        raise RuntimeError("Klar archive has no SHA-256 digest")
    got = hashlib.sha256(blob).hexdigest()
    if got != digest.split(":", 1)[1]:
        raise RuntimeError("Klar archive checksum mismatch")


def extract_klar_archive(blob: bytes, dest: Path) -> None:
    dest.mkdir(parents=True, exist_ok=True)
    with tarfile.open(fileobj=BytesIO(blob), mode="r:gz") as tar:
        tar.extraction_filter = getattr(tarfile, "data_filter", tarfile.tar_filter)
        wanted = pick_klar_member(
            [item.name for item in tar.getmembers() if item.isfile()]
        )
        if wanted is None:
            raise RuntimeError("Klar archive has no klar binary")
        member = tar.getmember(wanted)
        extracted = tar.extractfile(member)
        if extracted is None:
            raise RuntimeError("Klar archive could not be read")
        binary = dest / "klar"
        binary.write_bytes(extracted.read())
        binary.chmod(0o755)
        _extract_ui(tar, dest)


def _extract_ui(tar: tarfile.TarFile, dest: Path) -> None:
    ui = dest / "ui"
    if ui.exists():
        shutil.rmtree(ui)
    for item in tar.getmembers():
        if not item.isfile():
            continue
        path = Path(item.name)
        if ".." in path.parts or not path.parts or path.parts[0] != "ui":
            continue
        out = dest.joinpath(*path.parts)
        out.parent.mkdir(parents=True, exist_ok=True)
        fileobj = tar.extractfile(item)
        if fileobj is not None:
            out.write_bytes(fileobj.read())
