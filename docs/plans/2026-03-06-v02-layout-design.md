# ccline v0.2 Layout Redesign

## Problem

The v0.1 layout (`user@host /full/path branch*`) wastes space on low-value info (user@host, full path) and ignores useful data from Claude Code's JSON payload (model, tokens, cost). On narrow terminals the status line truncates, hiding the important bits.

## New Layout

```
Opus | tinnet/ccline | main* | 42k tks | $0.12
```

| Segment | Content | Color (Monokai Pro ~60%) | Source |
|---------|---------|--------------------------|--------|
| Model | `Opus` | muted green `#7a9e56` | `model.display_name` |
| Path | `tinnet/ccline` | muted cyan `#5a9ea0` | Last 2 components of `workspace.current_dir` |
| Git | `main*` | muted purple `#7a6db0` | `git2` branch + dirty |
| Tokens | `42k tks` | muted yellow `#b09a42` | `context_window.total_input_tokens + total_output_tokens` |
| Cost | `$0.12` | muted pink `#b04a60` | `cost.total_cost_usd` |
| Separators | `\|` | dark gray | `\x1b[90m` |

## What Changed from v0.1

- Removed `user@host` prefix
- Path shortened to last 2 directory components (Go/GitHub convention)
- Added model name, token count, session cost
- Colors switched from basic ANSI to muted Monokai Pro true-color (24-bit RGB)
- Pipe separators between segments

## Color Details

24-bit RGB escapes (`\x1b[38;2;r;g;bm`). Monokai Pro palette at ~60% brightness. Claude Code wraps the status line in Ink's `dimColor: true`, which further mutes the colors. This produces a subtle, non-distracting look that adapts to the terminal's light/dark theme.

## Data Flow

JSON on stdin from Claude Code (full schema: https://code.claude.com/docs/en/statusline#available-data):

```rust
#[derive(Deserialize)]
struct Input {
    workspace: Workspace,
    model: Option<Model>,
    cost: Option<Cost>,
    context_window: Option<ContextWindow>,
}

#[derive(Deserialize)]
struct Workspace { current_dir: String }

#[derive(Deserialize)]
struct Model { display_name: String }

#[derive(Deserialize)]
struct Cost { total_cost_usd: f64 }

#[derive(Deserialize)]
struct ContextWindow {
    total_input_tokens: u64,
    total_output_tokens: u64,
}
```

All new fields are `Option<T>`. Missing segments (e.g. before first API call) are skipped along with their separators.

## Token Formatting

- `< 1000` -> `"847 tks"`
- `1000..9999` -> `"1.2k tks"`
- `10000..999999` -> `"42k tks"`
- `>= 1000000` -> `"1.5M tks"`

No external crate needed — a simple `if/else` with `format!`.

## Cost Formatting

Always 2 decimal places: `$0.01`, `$1.23`, `$12.34`.

## Path Shortening

Split `current_dir` on `/`, take last 2 components, join with `/`.
Edge case: fewer than 2 components -> show whole path.

## Other Changes

- **README:** Add claude-powerline as "See also" link
- **bench.sh:** Update sample JSON and bash comparison script to match new format
- **Tests:** Update for new output format, add unit tests for `human_tokens()` and path shortening
