#!/usr/bin/env bash
# Run one opencode agent as a VISIBLE job. The job itself runs inside a cmux
# split beside this session; this script waits for its sentinel and prints the
# distilled reply. Outside cmux the job still runs, unseen.
# Usage: scripts/run-agent.sh <builder|reviewer|tester|scout> <prompt-file>
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
model_spec="$model"
case "$model" in
  *"#"*) ;;
  *) [ -z "$variant" ] || model_spec="$model#$variant" ;;
esac
repo="$(git rev-parse --show-toplevel)"
prompt_file="$(cd "$(dirname "$prompt_file")" && pwd)/$(basename "$prompt_file")"
jobs="$repo/.kh-lang/jobs"
events="$jobs/ai-$role.jsonl"
errors="$jobs/ai-$role.err"
reply="$jobs/ai-$role.txt"
done_file="$jobs/ai-$role.done"
wrapper="$jobs/run-$role.sh"
mkdir -p "$jobs"
rm -f "$done_file" "$errors" "$reply"

cat > "$wrapper" <<EOF
#!/usr/bin/env bash
cd "$repo"
export PATH="\$HOME/.cargo/bin:\$PATH"
perl -e 'alarm shift; exec @ARGV or die' "\${AI_TIMEOUT:-900}" \\
  opencode run -m "$model_spec" --agent "$role" --format json "\$(cat "$prompt_file")" \\
  > "$events" 2> "$errors"
echo "\$?" > "$done_file"
EOF
chmod +x "$wrapper"

surface=""
cleanup() {
  [ -z "$surface" ] || cmux close-surface --surface "$surface" --force > /dev/null 2>&1 || true
}
trap cleanup EXIT

if command -v cmux > /dev/null 2>&1; then
  surface="$(cmux new-split right --command "bash '$wrapper'" 2> /dev/null \
    | grep -oE '^OK surface:[0-9]+' | cut -d ' ' -f 2 || true)"
  [ -n "$surface" ] || echo "Couldn't open a cmux split; the job runs unseen." >&2
fi

if [ -n "$surface" ]; then
  waited=0
  limit=$(( ${AI_TIMEOUT:-900} + 60 ))
  while [ ! -f "$done_file" ]; do
    [ "$waited" -lt "$limit" ] || { echo "Job timed out after ${limit}s" >&2; exit 1; }
    sleep 2; waited=$((waited + 2))
  done
  status="$(cat "$done_file" 2>/dev/null || echo 1)"
else
  bash "$wrapper"
  status="$(cat "$done_file" 2>/dev/null || echo 1)"
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
