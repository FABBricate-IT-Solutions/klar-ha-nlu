#!/usr/bin/env bash
# Publish the staging app to its own store branch with the RC version.
# Supervisor offers an update whenever the version changes, so a fixed
# `version: "staging"` on main never updates. Add the repository once as
# https://github.com/FABBricate-IT-Solutions/klar-ha-nlu#staging-store
set -euo pipefail

STORE_BRANCH="${STORE_BRANCH:-staging-store}"
APP_DIR="klar_nlu_staging"
GIT_AUTHOR_NAME="github-actions[bot]"
GIT_AUTHOR_EMAIL="41898282+github-actions[bot]@users.noreply.github.com"

valid_version() {
  [[ "$1" =~ ^[0-9]+(\.[0-9]+)+-staging\.[0-9a-f]{7,40}$ ]]
}

render_store() {
  local version="$1" out="$2" root="$3"
  mkdir -p "$out/$APP_DIR"
  sed 's/^name: .*/name: Klar NLU (Staging)/' "$root/repository.yaml" >"$out/repository.yaml"
  for file in config.yaml DOCS.md CHANGELOG.md; do
    cp "$root/addon-staging/$file" "$out/$APP_DIR/$file"
  done
  sed -i "s/^version: .*/version: \"${version}\"/" "$out/$APP_DIR/config.yaml"
}

publish() {
  local version="$1" root work
  root="$(git rev-parse --show-toplevel)"
  work="$(mktemp -d)"
  # shellcheck disable=SC2064 # expand now: the locals are gone when EXIT fires
  trap "git -C $(printf %q "$root") worktree remove --force $(printf %q "$work") >/dev/null 2>&1 || true" EXIT

  if git ls-remote --exit-code --heads origin "$STORE_BRANCH" >/dev/null; then
    git fetch --depth=1 origin "$STORE_BRANCH"
    git worktree add --detach "$work" FETCH_HEAD
  else
    git worktree add --detach "$work" HEAD
    git -C "$work" checkout --orphan "$STORE_BRANCH-new"
  fi
  git -C "$work" rm -rqf --ignore-unmatch .
  render_store "$version" "$work" "$root"

  git -C "$work" add -A
  if git -C "$work" diff --cached --quiet; then
    echo "${STORE_BRANCH} already at ${version}"
    return
  fi
  git -C "$work" -c user.name="$GIT_AUTHOR_NAME" -c user.email="$GIT_AUTHOR_EMAIL" \
    commit -qm "staging app ${version}"
  git -C "$work" push origin "HEAD:refs/heads/${STORE_BRANCH}"
  echo "${STORE_BRANCH} -> ${version}"
}

self_test() {
  local root out
  root="$(git rev-parse --show-toplevel)"
  out="$(mktemp -d)"
  valid_version "2026.10.2-staging.3872503"
  ! valid_version "staging" || exit 1
  ! valid_version "2026.10.2" || exit 1
  ! valid_version "2026.10.2-staging.3872503; rm -rf /" || exit 1
  render_store "2026.10.2-staging.3872503" "$out" "$root"
  grep -qx 'version: "2026.10.2-staging.3872503"' "$out/$APP_DIR/config.yaml"
  grep -qx 'slug: klar_nlu_staging' "$out/$APP_DIR/config.yaml"
  grep -q '^image: ' "$out/$APP_DIR/config.yaml"
  grep -qx 'name: Klar NLU (Staging)' "$out/repository.yaml"
  [[ ! -e "$out/$APP_DIR/Dockerfile" && ! -e "$out/$APP_DIR/build.yaml" ]]
  rm -rf "$out"
  echo "ok"
}

main() {
  if [[ "${1:-}" == "--self-test" ]]; then
    self_test
    return
  fi
  local version="${1:-}"
  version="${version#v}"
  if ! valid_version "$version"; then
    echo "usage: publish-staging-store.sh <CalVer>-staging.<sha>  (or --self-test)" >&2
    exit 1
  fi
  publish "$version"
}

main "$@"
