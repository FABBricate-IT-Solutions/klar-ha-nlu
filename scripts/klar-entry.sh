#!/bin/sh
# Thin wrapper for environments that still ship a shell. Add-on options are
# loaded inside klar via serde (A-14); this script only fills default listen flags.
set -e

has_flag() {
  flag=$1
  shift
  for arg in "$@"; do
    [ "$arg" = "$flag" ] && return 0
  done
  return 1
}

prefix=""
has_flag --http "$@" || prefix="$prefix --http 0.0.0.0:10520"
has_flag --wyoming "$@" || prefix="$prefix --wyoming 0.0.0.0:10500"
has_flag --config-dir "$@" || prefix="$prefix --config-dir /config"
has_flag --data-dir "$@" || prefix="$prefix --data-dir /data"

# prefix is only our flags; values have no spaces
# shellcheck disable=SC2086
exec /usr/local/bin/klar $prefix "$@"
