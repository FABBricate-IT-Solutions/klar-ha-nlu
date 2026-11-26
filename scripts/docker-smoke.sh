#!/usr/bin/env bash
# Start a Klar image, wait for /api/v2/health, stop with SIGTERM, require a clean exit.
set -euo pipefail

image="${1:?usage: docker-smoke.sh <image> [name]}"
name="${2:-klar-smoke-$$}"
port_http="${KLAR_SMOKE_HTTP_PORT:-10520}"

cleanup() {
  docker rm -f "$name" >/dev/null 2>&1 || true
}
trap cleanup EXIT

docker run -d --name "$name" \
  -p "127.0.0.1:${port_http}:10520" \
  "$image"

ok=0
for _ in $(seq 1 60); do
  if curl -sf "http://127.0.0.1:${port_http}/api/v2/health" | grep -q '"ok":true'; then
    ok=1
    break
  fi
  sleep 1
done
if [[ "$ok" != "1" ]]; then
  docker logs "$name" >&2 || true
  echo "container never became healthy" >&2
  exit 1
fi

# docker stop sends SIGTERM, then SIGKILL after the grace period.
docker stop -t 15 "$name" >/dev/null
status="$(docker inspect -f '{{.State.Status}}' "$name")"
code="$(docker inspect -f '{{.State.ExitCode}}' "$name")"
if [[ "$status" != "exited" ]]; then
  echo "expected exited after SIGTERM, got ${status}" >&2
  exit 1
fi
case "$code" in
  0 | 143) ;;
  *)
    echo "unexpected exit code after SIGTERM: ${code}" >&2
    exit 1
    ;;
esac
echo "smoke ok: SIGTERM stop exit=${code}"
