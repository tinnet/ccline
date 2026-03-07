#!/usr/bin/env bash
set -euo pipefail

SAMPLE_JSON='{"workspace":{"current_dir":"'"$(pwd)"'","project_dir":"'"$(pwd)"'","added_dirs":[]}}'

BASH_CMD='input=$(cat); cwd=$(echo "$input" | jq -r '"'"'.workspace.current_dir'"'"'); user=$(whoami); host=$(hostname -s); git_info=""; if git -C "$cwd" rev-parse --git-dir >/dev/null 2>&1; then branch=$(git -C "$cwd" --no-optional-locks branch --show-current 2>/dev/null || echo ""); if [ -n "$branch" ]; then if ! git -C "$cwd" --no-optional-locks diff --quiet 2>/dev/null || ! git -C "$cwd" --no-optional-locks diff --cached --quiet 2>/dev/null || [ -n "$(git -C "$cwd" --no-optional-locks ls-files --others --exclude-standard 2>/dev/null)" ]; then status="*"; else status=""; fi; git_info=" $(printf '"'"'\033[90m'"'"')${branch}$(printf '"'"'\033[0m'"'"')$(printf '"'"'\033[36m'"'"')${status}$(printf '"'"'\033[0m'"'"')"; fi; fi; printf "%s@%s $(printf '"'"'\033[34m'"'"')%s$(printf '"'"'\033[0m'"'"')%s" "$user" "$host" "$cwd" "$git_info"'

echo "Benchmarking with input: $SAMPLE_JSON"
echo ""

hyperfine \
    --warmup 3 \
    --runs 50 \
    --input <(echo "$SAMPLE_JSON") \
    --command-name "rust" "./target/release/ccline" \
    --command-name "bash" "bash -c '$BASH_CMD'"
