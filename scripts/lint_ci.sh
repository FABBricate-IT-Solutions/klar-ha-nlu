#!/usr/bin/env bash
# Lint the CI definition and the shell scripts:
#   shellcheck  every tracked shell script
#   actionlint  the workflows, with shellcheck on every `run:` block
#   zizmor      the workflows; fails on a finding of medium severity or higher
# Needs uv (https://docs.astral.sh/uv/). It fetches the tools at the versions pinned here.
set -euo pipefail
cd "$(dirname "$0")/.."

ACTIONLINT_PY=1.7.12.25
SHELLCHECK_PY=0.11.0.1
ZIZMOR=1.33.1

# Scripts by shebang, so one without the .sh suffix is not missed.
mapfile -t scripts < <(
  git ls-files -z | xargs -0 awk 'FNR == 1 { if ($0 ~ /^#!.*[\/ ](ba|da)?sh( |$)/) print FILENAME; nextfile }'
)
if [[ ${#scripts[@]} -eq 0 ]]; then
  echo "no shell scripts found; the search is broken" >&2
  exit 1
fi

echo "== shellcheck: ${scripts[*]}"
uvx --from "shellcheck-py==${SHELLCHECK_PY}" shellcheck "${scripts[@]}"

echo "== actionlint"
uvx --with "shellcheck-py==${SHELLCHECK_PY}" --from "actionlint-py==${ACTIONLINT_PY}" actionlint

echo "== zizmor"
uvx "zizmor==${ZIZMOR}" --offline --min-severity medium .github/workflows
