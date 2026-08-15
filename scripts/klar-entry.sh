#!/bin/sh
set -e
if [ "$#" -gt 0 ]; then
  exec /usr/local/bin/klar "$@"
fi
token=""
if [ -f /data/options.json ]; then
  token=$(sed -n 's/.*"token"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' /data/options.json | head -1)
fi
if [ -n "$token" ]; then
  exec /usr/local/bin/klar --http 0.0.0.0:10520 --wyoming 0.0.0.0:10500 --config-dir /config --data-dir /data --token "$token"
fi
exec /usr/local/bin/klar --http 0.0.0.0:10520 --wyoming 0.0.0.0:10500 --config-dir /config --data-dir /data --token-file /data/klar_token
