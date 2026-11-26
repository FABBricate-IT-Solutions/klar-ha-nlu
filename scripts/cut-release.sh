#!/usr/bin/env bash
# Cut a CalVer release without pushing a new commit to protected main.
# New bumps go through chore/release-* PRs (same pattern as 2026.8.46–52).
set -euo pipefail

REQUIRED_CHECKS=(
  test
  clippy
  rustfmt
  web
  release-gates
  cargo-audit
  cargo-deny
  gitleaks
  hassfest
  hacs
)

GIT_AUTHOR_NAME="github-actions[bot]"
GIT_AUTHOR_EMAIL="41898282+github-actions[bot]@users.noreply.github.com"

release_from_message() {
  local msg="$1"
  local tag=""
  tag="$(sed -n 's/^chore(release): prepare for \(v\?[0-9][0-9.]*\).*/\1/p' <<<"$msg" | head -1)"
  tag="${tag#v}"
  if [[ -z "$tag" ]]; then
    tag="$(sed -n 's#.*chore/release-\(v\?[0-9][0-9.]*\).*#\1#p' <<<"$msg" | head -1)"
    tag="${tag#v}"
  fi
  printf '%s' "$tag"
}

git_ident() {
  git -c "user.name=${GIT_AUTHOR_NAME}" -c "user.email=${GIT_AUTHOR_EMAIL}" "$@"
}

write_tag_output() {
  local tag="$1"
  if [[ -z "${GITHUB_OUTPUT:-}" ]]; then
    echo "GITHUB_OUTPUT is unset" >&2
    exit 1
  fi
  echo "tag=${tag}" >> "$GITHUB_OUTPUT"
}

tag_and_exit() {
  local tag="$1"
  local sha="${2:-HEAD}"
  if git rev-parse "refs/tags/${tag}" >/dev/null 2>&1; then
    echo "Tag ${tag} already exists — skip publish from this push"
    write_tag_output ""
    exit 0
  fi
  git_ident tag -a "$tag" -m "$tag" "$sha"
  git push origin "refs/tags/${tag}"
  write_tag_output "$tag"
  exit 0
}

write_changelog() {
  local tag="$1"
  TAG="$tag" python3 - <<'PY'
import os
import subprocess
from pathlib import Path

tag = os.environ["TAG"].lstrip("v")
generated = subprocess.check_output(["git-cliff", "--tag", tag], text=True)
start = generated.find(f"## [{tag}]")
if start < 0:
    raise SystemExit(f"git-cliff produced no heading for {tag}")
nxt = generated.find("\n## [", start + 1)
section = generated[start : nxt if nxt >= 0 else None].rstrip() + "\n\n"
path = Path("CHANGELOG.md")
existing = path.read_text()
body_at = existing.find("## [")
if body_at < 0:
    raise SystemExit("CHANGELOG.md has no version heading")
path.write_text(existing[:body_at] + section + existing[body_at:])
PY
}

check_conclusion() {
  local json="$1"
  local name="$2"
  jq -r --arg n "$name" '
    [.statusCheckRollup[]? | select((.name == $n) or (.name | endswith(" / " + $n)))]
    | if length == 0 then "missing"
      else
        .[-1] as $c
        | (
            ($c.conclusion // "") as $conclusion
            | ($c.state // "") as $state
            | ($c.status // "") as $status
            | if ($conclusion | length) > 0 then $conclusion
              elif ($state | length) > 0 then $state
              elif ($status | length) > 0 then $status
              else "pending"
              end
          )
          | ascii_downcase
      end
  ' <<<"$json"
}

wait_required_checks() {
  local pr="$1"
  local deadline=$((SECONDS + 3600))
  while (( SECONDS < deadline )); do
    local json status failed=0 pending=0 name conclusion
    json="$(gh pr view "$pr" --json mergeStateStatus,statusCheckRollup)"
    status="$(jq -r '.mergeStateStatus' <<<"$json")"
    echo "mergeStateStatus=${status}"
    for name in "${REQUIRED_CHECKS[@]}"; do
      conclusion="$(check_conclusion "$json" "$name")"
      echo "${name}: ${conclusion}"
      case "$conclusion" in
        success|skipped|neutral|pass) ;;
        failure|cancelled|canceled|timed_out|stale|action_required|error|fail)
          failed=1
          ;;
        *)
          pending=1
          ;;
      esac
    done
    if (( failed )); then
      echo "Required check failed on PR ${pr}" >&2
      gh pr checks "$pr" || true
      exit 1
    fi
    if (( pending == 0 )); then
      case "$status" in
        CLEAN | HAS_HOOKS | BEHIND | UNSTABLE)
          return 0
          ;;
        BLOCKED)
          echo "Required checks are green but merge is BLOCKED (review or rule). Not using --admin." >&2
          gh pr view "$pr" --json url,reviewDecision,mergeStateStatus
          exit 1
          ;;
        DIRTY)
          echo "PR ${pr} has conflicts" >&2
          exit 1
          ;;
      esac
    fi
    sleep 20
  done
  echo "Timed out waiting for the 10 required checks on PR ${pr}" >&2
  gh pr checks "$pr" || true
  exit 1
}

open_or_update_pr() {
  local tag="$1"
  local branch="$2"
  local pr=""
  pr="$(gh pr list --head "$branch" --base main --state open --json number --jq '.[0].number // empty')"
  if [[ -z "$pr" ]]; then
    local url
    url="$(gh pr create \
      --base main \
      --head "$branch" \
      --title "chore(release): prepare for ${tag}" \
      --body "$(cat <<EOF
Prepares **${tag}**.

Order (package 5.1): the cut job waits for the 10 required checks and tags this branch tip. **Build** then builds both arches, smokes each image with SIGTERM, scans, pushes images, writes the manifest, and creates the GitHub Release. Only after that succeeds does the land job merge this PR (so the add-on version on \`main\` never appears before its images).

## Test plan
- [ ] Required checks green: test, clippy, rustfmt, web, release-gates, cargo-audit, cargo-deny, gitleaks, hassfest, hacs
- [ ] Tag ${tag} points at this branch tip before merge
- [ ] GHCR images and the GitHub Release exist before this PR merges
- [ ] Merge lands on main without a direct push (no \`--admin\`)
EOF
)")"
    echo "$url" >&2
    pr="${url##*/}"
    if [[ ! "$pr" =~ ^[0-9]+$ ]]; then
      pr="$(gh pr list --head "$branch" --base main --state open --json number --jq '.[0].number // empty')"
    fi
  else
    echo "Updating existing release PR #${pr}" >&2
  fi
  if [[ -z "$pr" ]]; then
    echo "Could not open or find a release PR for ${branch}" >&2
    echo "Compare: https://github.com/${GITHUB_REPOSITORY}/compare/main...${branch}" >&2
    exit 1
  fi
  printf '%s' "$pr"
}

prepare_via_pr() {
  # Bump on a branch, wait for required checks, tag the tip. Do not merge yet:
  # landing addon/config.yaml on main is what Supervisor reads (V-02).
  local tag="$1"
  local branch="chore/release-${tag}"
  git checkout -B "$branch"
  git add Cargo.toml Cargo.lock addon/config.yaml \
    custom_components/klar_nlu/manifest.json \
    custom_components/klar_nlu/const.py CHANGELOG.md \
    addon/CHANGELOG.md addon-staging/CHANGELOG.md
  if git diff --cached --quiet; then
    echo "Release files already match ${tag}"
  else
    git_ident commit -m "chore(release): prepare for ${tag}"
  fi
  git push --force-with-lease origin "HEAD:refs/heads/${branch}"

  local pr sha
  pr="$(open_or_update_pr "$tag" "$branch")"
  echo "Release PR: https://github.com/${GITHUB_REPOSITORY}/pull/${pr}"
  wait_required_checks "$pr"
  local merge_status
  merge_status="$(gh pr view "$pr" --json mergeStateStatus --jq .mergeStateStatus)"
  if [[ "$merge_status" == "BLOCKED" ]]; then
    echo "mergeStateStatus=BLOCKED; refusing --admin merge" >&2
    exit 1
  fi
  sha="$(gh pr view "$pr" --json headRefOid --jq .headRefOid)"
  if [[ -z "$sha" ]]; then
    echo "PR ${pr} has no head SHA" >&2
    exit 1
  fi
  echo "pr=${pr}" >> "$GITHUB_OUTPUT"
  git fetch origin "$sha"
  tag_and_exit "$tag" "$sha"
}

land_addon() {
  # Called only after Build has pushed images, scanned, and created the Release.
  local tag="${TAG:-}"
  tag="${tag#v}"
  if [[ -z "$tag" ]]; then
    echo "usage: cut-release.sh --land-addon  (with TAG set)" >&2
    exit 1
  fi
  local branch="chore/release-${tag}"
  local pr merge_status
  pr="$(gh pr list --head "$branch" --base main --state open --json number --jq '.[0].number // empty')"
  if [[ -z "$pr" ]]; then
    pr="$(gh pr list --head "$branch" --base main --state merged --json number --jq '.[0].number // empty')"
    if [[ -n "$pr" ]]; then
      echo "Release PR #${pr} already merged — add-on version is on main"
      exit 0
    fi
    echo "No release PR for ${branch}" >&2
    exit 1
  fi
  merge_status="$(gh pr view "$pr" --json mergeStateStatus --jq .mergeStateStatus)"
  if [[ "$merge_status" == "BLOCKED" ]]; then
    echo "mergeStateStatus=BLOCKED; refusing --admin merge" >&2
    exit 1
  fi
  gh pr merge "$pr" --merge --delete-branch
  echo "Landed add-on version ${tag} via PR #${pr}"
}

self_test() {
  [[ "$(release_from_message $'chore(release): prepare for 2026.8.53')" == "2026.8.53" ]]
  [[ "$(release_from_message $'chore(release): prepare for v2026.8.1')" == "2026.8.1" ]]
  [[ "$(release_from_message $'Merge pull request #146 from org/chore/release-2026.8.52\n\nchore(release): prepare for 2026.8.52')" == "2026.8.52" ]]
  [[ "$(release_from_message $'Merge pull request #146 from FABBricate-IT-Solutions/chore/release-2026.8.52')" == "2026.8.52" ]]
  [[ -z "$(release_from_message $'Merge pull request #147 from org/feat/conversation-suite')" ]]
  grep -q '^prepare_via_pr() {$' "$0"
  grep -q '^land_addon() {$' "$0"
  if grep -q '^land_via_pr() {$' "$0"; then
    echo "old land_via_pr function must stay removed" >&2
    exit 1
  fi
  echo "ok"
}

main() {
  if [[ "${1:-}" == "--self-test" ]]; then
    self_test
    return
  fi

  if [[ "${1:-}" == "--land-addon" ]]; then
    if [[ -z "${GITHUB_REPOSITORY:-}" ]]; then
      echo "usage: cut-release.sh --land-addon  (run from Release land job)" >&2
      exit 1
    fi
    land_addon
    return
  fi

  if [[ -z "${GITHUB_OUTPUT:-}" || -z "${GITHUB_REPOSITORY:-}" ]]; then
    echo "usage: cut-release.sh --self-test|--land-addon  (or run from Release with GITHUB_OUTPUT set)" >&2
    exit 1
  fi

  local existing version tag
  existing="$(release_from_message "${MESSAGE:-}")"
  if [[ -n "$existing" && -z "${VERSION:-}" ]]; then
    tag_and_exit "$existing"
  fi

  if [[ -n "${VERSION:-}" ]]; then
    version="$VERSION"
  else
    version="$(python3 scripts/bump-version.py next)"
  fi
  version="$(tr -d '[:space:]' <<<"$version")"
  version="${version#v}"
  tag="$version"
  if git rev-parse "refs/tags/${tag}" >/dev/null 2>&1; then
    echo "Tag ${tag} already exists — nothing to release" >&2
    exit 1
  fi

  python3 scripts/bump-version.py "$version"
  write_changelog "$tag"
  python3 scripts/release-notes.py --sync-addons
  prepare_via_pr "$tag"
}

main "$@"
