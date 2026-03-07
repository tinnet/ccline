#!/bin/sh
# POSIX shell equivalent of the ccline Rust binary.
# Reads Claude Code status JSON from stdin, outputs ANSI-formatted status line.
set -eu

input=$(cat)
cwd=$(echo "$input" | jq -r ".workspace.current_dir")
model=$(echo "$input" | jq -r ".model.display_name")
cost=$(echo "$input" | jq -r ".cost.total_cost_usd")
in_tks=$(echo "$input" | jq -r ".context_window.total_input_tokens")
out_tks=$(echo "$input" | jq -r ".context_window.total_output_tokens")
total_tks=$((in_tks + out_tks))
pct=$(echo "$input" | jq -r ".context_window.used_percentage")
win=$(echo "$input" | jq -r ".context_window.context_window_size")
last_two=$(echo "$cwd" | rev | cut -d/ -f1-2 | rev)

human_tokens() {
  n=$1
  if [ "$n" -ge 1000000 ]; then
    awk "BEGIN{printf \"%.1fM\", $n/1000000}"
  elif [ "$n" -ge 10000 ]; then
    echo "$((n / 1000))k"
  elif [ "$n" -ge 1000 ]; then
    awk "BEGIN{printf \"%.1fk\", $n/1000}"
  else
    echo "$n"
  fi
}

tks_fmt=$(human_tokens "$total_tks")
win_fmt=$(human_tokens "$win")
pct_fmt=$(printf '%.0f' "$pct")
cost_fmt=$(printf '$%.2f' "$cost")

# Colors (Monokai Pro ~60%)
GREEN="\033[38;2;122;158;86m"
CYAN="\033[38;2;90;158;160m"
PURPLE="\033[38;2;122;109;176m"
YELLOW="\033[38;2;176;154;66m"
LGRAY="\033[37m"
GRAY="\033[90m"
RST="\033[0m"

SEP=" ${GRAY}|${RST} "

git_info=""
if git -C "$cwd" rev-parse --git-dir >/dev/null 2>&1; then
  branch=$(git -C "$cwd" --no-optional-locks branch --show-current 2>/dev/null || echo "")
  if [ -n "$branch" ]; then
    if ! git -C "$cwd" --no-optional-locks diff --quiet 2>/dev/null || \
       ! git -C "$cwd" --no-optional-locks diff --cached --quiet 2>/dev/null || \
       [ -n "$(git -C "$cwd" --no-optional-locks ls-files --others --exclude-standard 2>/dev/null)" ]; then
      dirty="*"
    else
      dirty=""
    fi
    git_info="${SEP}${PURPLE}${branch}${dirty}${RST}"
  fi
fi

printf "${GREEN}${model}${RST}${SEP}${CYAN}${last_two}${RST}${git_info}${SEP}${YELLOW}${pct_fmt}%%/${win_fmt} ctx${RST}${SEP}${LGRAY}${tks_fmt}/${cost_fmt} tks${RST}"
