# ccline

A fast [status line](https://code.claude.com/docs/en/statusline) for [Claude Code](https://docs.anthropic.com/en/docs/claude-code).

**This is a personal tool.** It's published as a starting point for forking and customizing, not as a general-purpose library. The layout is hardcoded to my preferences — fork it and make it yours.

![example output](docs/example.svg)

## Why speed matters

Claude Code's status line command [runs on every prompt refresh](https://code.claude.com/docs/en/statusline) (300ms debounce). A shell script spawns multiple processes per invocation (bash, jq, git), and popular status line tools take 120–800ms. This Rust binary uses native libraries (libgit2, serde) to do the same work in ~16ms — over 2x faster than an equivalent shell script and 8–50x faster than the alternatives.

It's also kinder to your battery. Over a full day of heavy coding, those saved milliseconds add up to less CPU time and less heat. Napkin math puts the annual energy savings at a few cents at Hydro-Québec rates. The API call that just answered your prompt probably used more electricity, but at least *your* fan stays quiet.

## Fork and customize

The layout is hardcoded in `src/main.rs` (~80 lines). There's no config file by design — editing source and running `cargo build --release` is faster than parsing config on every invocation.

Claude Code sends a [rich JSON payload](https://code.claude.com/docs/en/statusline#available-data) on stdin with every refresh. This project currently uses only a subset:

| Used | Available but unused |
|------|---------------------|
| `workspace.current_dir` | `model.id` |
| `model.display_name` | `cost.total_lines_added/removed` |
| `cost.total_cost_usd` | `vim.mode`, `session_id`, `worktree.*` |
| `effort.level` | `context_window.total_input_tokens/total_output_tokens` |
| `context_window.context_window_size` | |
| `context_window.used_percentage` | |
| (git via libgit2) | |

Fork this repo and add the fields that matter to you. The serde structs in `main.rs` are easy to extend.

## Install

With [mise](https://mise.jdx.dev):

```bash
mise use -g github:tinnet/ccline
```

Or from source:

```bash
git clone https://github.com/tinnet/ccline.git
cd ccline
cargo install --path .
```

Then add to `~/.claude/settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "ccline"
  }
}
```

## Current layout

| Segment | Color (Monokai Pro ~60%) | Source |
|---------|--------------------------|--------|
| Model | muted green `#7a9e56` | `model.display_name` + `effort.level` (muted yellow) |
| Path | muted cyan `#5a9ea0` | Last 2 of `workspace.current_dir` |
| Git | muted purple `#7a6db0` | `git2` branch + dirty |
| Context | muted yellow `#b09a42` | `used_percentage`/`context_window_size` |
| Cost | light gray | `cost.total_cost_usd` |
| Separators | dark gray | `\x1b[90m` |

## Benchmarking

```bash
mise run bench
```

Compares the Rust binary against a POSIX shell equivalent and other status line tools using [hyperfine](https://github.com/sharkdp/hyperfine).

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `ccline` | 15.7 ± 1.2 | 12.4 | 22.0 | 1.00 |
| `ccline.sh (via bash)` | 36.8 ± 1.5 | 32.2 | 40.3 | 2.34 ± 0.20 |
| `ccline.sh (via sh)` | 38.6 ± 1.4 | 34.1 | 41.4 | 2.45 ± 0.21 |
| `ccline.sh (via zsh)` | 39.2 ± 1.3 | 35.6 | 43.5 | 2.49 ± 0.21 |
| `starship-claude (defaults)` | 124.1 ± 5.1 | 118.0 | 132.1 | 7.89 ± 0.69 |
| `claude-powerline (defaults)` | 213.7 ± 4.5 | 206.6 | 221.8 | 13.59 ± 1.09 |
| `ccstatusline (defaults)` | 814.1 ± 14.0 | 797.2 | 838.0 | 51.75 ± 4.09 |

Versions: claude-powerline 1.32.0, ccstatusline 2.2.30, starship-claude (upstream `main`, vendored in `bench/`) with starship 1.26.0. Competitor timings include ~55ms of `mise x` startup overhead.

Optimizations:
- `ccline.sh`: single jq call (inspired by [starship-claude](https://github.com/martinemde/starship-claude)) — 169ms → 111ms (-34%)

## See also

- [Owloops/claude-powerline](https://github.com/Owloops/claude-powerline) — a feature-rich, configurable status line plugin
- [sirmalloc/ccstatusline](https://github.com/sirmalloc/ccstatusline) — a Node.js status line with built-in themes
- [martinemde/starship-claude](https://github.com/martinemde/starship-claude) — bridges Claude Code data into a Starship prompt

## License

MIT

---

Made with ❤️ in Montreal
