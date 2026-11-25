#!/usr/bin/env bash
# Run every generator and fail when the working tree differs afterwards, anywhere:
# packs, datasets, test sources and the Home Assistant translations alike.
# Run it on a committed tree; needs python3 and cargo (rustfmt).
set -euo pipefail
cd "$(dirname "$0")/.."

# scripts/gen_familienhaus_de.py is left out on purpose: tests/datasets/familienhaus_de
# has been edited by hand since and the generator no longer reproduces it.
generators=(
  scripts/lang_packs/generate.py
  scripts/gen_voice_suite.py
  scripts/parity/generate.py
  scripts/full_home/generate.py
)

echo "== cargo run --bin export_web_contract"
cargo run --locked --bin export_web_contract >/dev/null

state() {
  git status --porcelain
  git diff
}

before="$(state)"
if [[ -n "${CI:-}" && -n "$before" ]]; then
  echo "the tree is not clean before the generators run:" >&2
  git status --porcelain >&2
  exit 1
fi

for generator in "${generators[@]}"; do
  echo "== ${generator}"
  python3 "${generator}" >/dev/null
done
echo "== cargo fmt --all"
cargo fmt --all

if [[ "$(state)" != "$before" ]]; then
  git status --porcelain
  git --no-pager diff --stat
  {
    echo "generated files are stale. Run and commit:"
    printf '  python3 %s\n' "${generators[@]}"
    echo "  cargo fmt --all"
  } >&2
  exit 1
fi
echo "generators are a no-op"
