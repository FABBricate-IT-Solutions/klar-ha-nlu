#!/usr/bin/env bash
# Open (or refresh) a PR that promotes staging into main. Never merges.
set -euo pipefail

BRANCH="release/promote-staging"
TITLE="chore(promote): staging into main"

self_test() {
  grep -q '^BRANCH="release/promote-staging"$' "$0"
  grep -q 'gh pr create' "$0"
  # Only the executable path matters; this comment must not say the gh merge words.
  if awk '/^main\(\)/,/^}$/' "$0" | grep -qE 'gh pr merge'; then
    echo "promote-staging.sh must not merge" >&2
    exit 1
  fi
  echo "ok"
}

main() {
  if [[ "${1:-}" == "--self-test" ]]; then
    self_test
    return
  fi

  if [[ -z "${GH_TOKEN:-}" || -z "${GITHUB_REPOSITORY:-}" ]]; then
    echo "usage: promote-staging.sh --self-test  (or run from Promote with GH_TOKEN set)" >&2
    exit 1
  fi

  git fetch origin staging main
  local ahead
  ahead="$(git rev-list --count origin/main..origin/staging)"
  if [[ "$ahead" -eq 0 ]]; then
    echo "staging has nothing new for main — skip"
    exit 0
  fi

  git checkout -B "$BRANCH" origin/staging
  git push --force-with-lease origin "HEAD:refs/heads/${BRANCH}"

  local pr
  pr="$(gh pr list --head "$BRANCH" --base main --state open --json number --jq '.[0].number // empty')"
  if [[ -z "$pr" ]]; then
    local url
    url="$(gh pr create \
      --base main \
      --head "$BRANCH" \
      --title "$TITLE" \
      --body "$(cat <<EOF
Scheduled promotion of \`staging\` into \`main\` (package 5.2).

\`staging\` is **${ahead}** commit(s) ahead of \`main\`. Review, wait for the required checks, then merge. The Release workflow cuts the next CalVer after the merge.

## Test plan
- [ ] Required checks green on this PR
- [ ] Diff is the intended staging backlog (no surprise version bumps)
- [ ] Merge with a normal merge commit (not \`--admin\`)
EOF
)")"
    echo "Opened ${url}"
  else
    echo "Promotion PR already open: https://github.com/${GITHUB_REPOSITORY}/pull/${pr}"
  fi
}

main "$@"
