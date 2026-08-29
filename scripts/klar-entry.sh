#!/bin/sh
set -e

has_flag() {
  flag=$1
  shift
  for arg in "$@"; do
    [ "$arg" = "$flag" ] && return 0
  done
  return 1
}

token=""
bundle=""
ui_locale=""
if [ -f /data/options.json ]; then
  token=$(sed -n 's/.*"token"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' /data/options.json | head -1)
  bundle=$(sed -n 's/.*"support_bundle"[[:space:]]*:[[:space:]]*\(true\|false\).*/\1/p' /data/options.json | head -1)
  ui_locale=$(sed -n 's/.*"ui_locale"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' /data/options.json | head -1)
fi

prefix=""
has_flag --http "$@" || prefix="$prefix --http 0.0.0.0:10520"
has_flag --wyoming "$@" || prefix="$prefix --wyoming 0.0.0.0:10500"
has_flag --config-dir "$@" || prefix="$prefix --config-dir /config"
has_flag --data-dir "$@" || prefix="$prefix --data-dir /data"
if [ -n "$token" ] && ! has_flag --token "$@"; then
  export KLAR_TOKEN="$token"
fi
if [ "$bundle" = "true" ]; then
  export KLAR_SUPPORT_BUNDLE=1
fi
if [ -n "$ui_locale" ]; then
  export KLAR_UI_LOCALE="$ui_locale"
fi

# prefix is only our flags; values have no spaces
# shellcheck disable=SC2086
exec /usr/local/bin/klar $prefix "$@"
