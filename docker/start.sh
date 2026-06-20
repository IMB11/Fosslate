#!/usr/bin/env sh
set -eu

export NODE_ENV="${NODE_ENV:-production}"
export PORT="${PORT:-3000}"
export HOSTNAME="${HOSTNAME:-0.0.0.0}"
export API_HOST="${API_HOST:-127.0.0.1}"
export API_PORT="${API_PORT:-4000}"
export INTERNAL_API_URL="${INTERNAL_API_URL:-http://127.0.0.1:4000}"

API_PID=""
WEB_PID=""

cleanup() {
  trap - INT TERM EXIT

  if [ -n "$API_PID" ]; then
    kill "$API_PID" 2>/dev/null || true
  fi

  if [ -n "$WEB_PID" ]; then
    kill "$WEB_PID" 2>/dev/null || true
  fi

  wait 2>/dev/null || true
}

trap 'cleanup; exit 143' INT TERM
trap cleanup EXIT

/app/fosslate-api &
API_PID="$!"

NEXT_SERVER="${NEXT_SERVER:-/app/apps/web/server.js}"
if [ ! -f "$NEXT_SERVER" ] && [ -f /app/server.js ]; then
  NEXT_SERVER="/app/server.js"
fi

node "$NEXT_SERVER" &
WEB_PID="$!"

while :; do
  if ! kill -0 "$API_PID" 2>/dev/null; then
    set +e
    wait "$API_PID"
    STATUS="$?"
    set -e
    cleanup
    exit "$STATUS"
  fi

  if ! kill -0 "$WEB_PID" 2>/dev/null; then
    set +e
    wait "$WEB_PID"
    STATUS="$?"
    set -e
    cleanup
    exit "$STATUS"
  fi

  sleep 1 &
  wait "$!"
done

