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
last_two=$(echo "$cwd" | rev | cut -d/ -f1-2 | rev)

# Token formatting
if [ "$total_tks" -ge 1000000 ]; then
  tks=$(awk "BEGIN{printf \"%.1fM tks\", $total_tks/1000000}")
elif [ "$total_tks" -ge 10000 ]; then
  tks="$((total_tks / 1000))k tks"
elif [ "$total_tks" -ge 1000 ]; then
  tks=$(awk "BEGIN{printf \"%.1fk tks\", $total_tks/1000}")
else
  tks="${total_tks} tks"
fi

cost_fmt=$(printf '$%.2f' "$cost")

# Colors (Monokai Pro ~60%)
GREEN="\033[38;2;122;158;86m"
CYAN="\033[38;2;90;158;160m"
PURPLE="\033[38;2;122;109;176m"
YELLOW="\033[38;2;176;154;66m"
PINK="\033[38;2;176;74;96m"
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

printf "${GREEN}${model}${RST}${SEP}${CYAN}${last_two}${RST}${git_info}${SEP}${YELLOW}${tks}${RST}${SEP}${PINK}${cost_fmt}${RST}"
