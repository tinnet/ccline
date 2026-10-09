#!/bin/sh
# POSIX shell equivalent of the ccline Rust binary.
# Reads Claude Code status JSON from stdin, outputs ANSI-formatted status line.
set -eu

eval "$(cat | jq -r '
  "cwd=\(.workspace.current_dir | @sh) " +
  "model=\(.model.display_name | @sh) " +
  "effort=\((.effort.level // "") | @sh) " +
  "cost=\(.cost.total_cost_usd) " +
  "pct=\((.context_window.used_percentage // "") | @sh) " +
  "win=\(.context_window.context_window_size)"
')"
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

cost_fmt=$(printf '$%.2f' "$cost")

# Colors (Monokai Pro ~60%)
GREEN="\033[38;2;122;158;86m"
CYAN="\033[38;2;90;158;160m"
PURPLE="\033[38;2;122;109;176m"
YELLOW="\033[38;2;176;154;66m"
RED="\033[38;2;176;67;94m"
ORANGE="\033[38;2;174;105;71m"
LGRAY="\033[37m"
GRAY="\033[90m"
RST="\033[0m"

SEP=" ${GRAY}|${RST} "

# Repo mark: shape x color picked by 32-bit FNV-1a of the repo name's UTF-8 bytes
repo_mark() {
  h=2166136261
  for b in $(printf '%s' "$1" | od -An -tu1); do
    h=$(( ((h ^ b) * 16777619) & 4294967295 ))
  done
  set -- ● ■ ▲ ▼ ◆ ★ ✚ ✦ ✿ ◐ ✱ ⬢
  shift $((h % 12))
  shape=$1
  set -- "$RED" "$ORANGE" "$YELLOW" "$GREEN" "$CYAN" "$PURPLE"
  shift $((h / 12 % 6))
  printf '%s' "$1${shape}${RST} "
}

mark=""
git_info=""
if git -C "$cwd" rev-parse --git-dir >/dev/null 2>&1; then
  toplevel=$(git -C "$cwd" rev-parse --show-toplevel 2>/dev/null || echo "")
  if [ -n "$toplevel" ]; then
    mark=$(repo_mark "$(basename "$toplevel")")
  fi
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

ctx_info=""
if [ -n "$pct" ]; then
  ctx_info="${SEP}${YELLOW}$(printf '%.0f' "$pct")%%/$(human_tokens "$win") ctx${RST}"
fi

effort_suffix=""
if [ -n "$effort" ]; then
  effort_suffix=" ${GRAY}(${RST}${YELLOW}${effort}${RST}${GRAY})${RST}"
fi

printf "${GREEN}${model}${RST}${effort_suffix}${SEP}${mark}${CYAN}${last_two}${RST}${git_info}${ctx_info}${SEP}${LGRAY}${cost_fmt}${RST}"
