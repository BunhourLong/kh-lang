#!/usr/bin/env bash
# Run one opencode agent as a VISIBLE job: once the run produces a session, a
# cmux split opens beside this one and attaches to the live run. The split and
# the per-job server wind down when the script exits.
# Usage: scripts/run-agent.sh <builder|reviewer|tester|scout> <prompt-file>
# Run from anywhere inside the repo. Outside cmux it still runs, unseen.
set -euo pipefail

role="${1:-}"
prompt_file="${2:-}"
case "$role" in
  builder|reviewer|tester|scout) ;;
  *) echo "Usage: scripts/run-agent.sh <builder|reviewer|tester|scout> <prompt-file>" >&2; exit 2 ;;
esac
[ -s "$prompt_file" ] || { echo "Prompt file is missing or empty: $prompt_file" >&2; exit 1; }

model="${AI_MODEL:-opencode/deepseek-v4.1-flash}"
variant="${AI_VARIANT:-max}"
repo="$(git rev-parse --show-toplevel)"
jobs="$repo/.kh-lang/jobs"
events="$jobs/ai-$role.jsonl"
errors="$jobs/ai-$role.err"
reply="$jobs/ai-$role.txt"
mkdir -p "$jobs"

url=""
surface=""
server=""
run=""
session=""
work="$(mktemp -d)"
cleanup() {
  [ -z "$surface" ] || cmux close-surface --surface "$surface" --force > /dev/null 2>&1 || true
  [ -z "$server" ] || kill "$server" 2> /dev/null || true
  [ -z "$run" ] || kill "$run" 2> /dev/null || true
  rm -rf "$work"
}
trap cleanup EXIT

# opencode hangs at provider rate limits instead of exiting; a surviving alarm stops it.
opencode_run() {
  local timeout="${AI_TIMEOUT:-900}"
  perl -e 'alarm shift; exec @ARGV or die' "$timeout" opencode run "$@"
}

watch_job() {
  local waited=0
  openssl rand -hex 16 > "$work/password"
  export OPENCODE_SERVER_PASSWORD="$(cat "$work/password")"
  (cd "$repo" && exec opencode serve > "$work/serve.log" 2>&1 < /dev/null) &
  server=$!
  disown "$server"
  until [ -n "$url" ] || [ "$waited" -ge 20 ]; do
    sleep 1; waited=$((waited + 1))
    url="$(grep -oE 'http://127\.0\.0\.1:[0-9]+' "$work/serve.log" | head -n 1 || true)"
  done
  [ -n "$url" ] || { echo "opencode serve didn't start; job runs headless." >&2; return 0; }
}

watch_job

open_split() {
  command -v cmux > /dev/null 2>&1 || return 0
  local waited=0
  while [ "$waited" -lt "${AI_SPLIT_WAIT:-15}" ]; do
    session="$(grep -m1 -oE '"sessionID":"[^"]+"' "$events" | head -n1 | cut -d'"' -f4 || true)"
    [ -n "$session" ] && break
    kill -0 "$run" 2> /dev/null || return 0
    sleep 1; waited=$((waited + 1))
  done
  [ -n "$session" ] || return 0
  surface="$(cmux new-split right --command "OPENCODE_SERVER_PASSWORD=$(cat "$work/password") opencode attach $url --dir $repo -s $session" 2> /dev/null \
    | grep -oE '^OK surface:[0-9]+' | cut -d ' ' -f 2 || true)"
  [ -n "$surface" ] || echo "Couldn't open a cmux split; the job runs unseen." >&2
}

status=0
if [ -n "$url" ]; then
  opencode_run --attach "$url" --dir "$repo" -m "$model" --variant "$variant" --agent "$role" --format json "$(cat "$prompt_file")" \
    > "$events" 2> "$errors" < /dev/null &
  run=$!
  open_split
  wait "$run" || status=$?
  run=""
else
  opencode_run --dir "$repo" -m "$model" --variant "$variant" --agent "$role" --format json "$(cat "$prompt_file")" \
    > "$events" 2> "$errors" < /dev/null || status=$?
fi

if [ "$status" -eq 142 ]; then
  echo "Stopped after ${AI_TIMEOUT:-900}s (provider stall). Check $errors" >&2
  exit 1
fi
api_error="$(grep -m1 '"type":"error"' "$events" 2>/dev/null | jq -r '.error.data.message // .error.name // empty' 2>/dev/null || true)"
if [ "$status" -ne 0 ] || [ -n "$api_error" ]; then
  echo "opencode failed: ${api_error:-exit $status}" >&2
  tail -n 5 "$errors" >&2 || true
  exit 1
fi

jq -r 'select(.type=="text") | .part.text' "$events" > "$reply" 2>/dev/null || true
[ -s "$reply" ] || { echo "opencode returned nothing" >&2; exit 1; }
jq -r --arg role "$role" 'select(.type=="step_finish") | .part | [now | todate, $role, .cost, .tokens.input, .tokens.output] | @tsv' \
  "$events" >> "$jobs/usage.tsv" 2> /dev/null || true
cat "$reply"
