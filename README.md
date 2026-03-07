# ccline

A fast [status line](https://code.claude.com/docs/en/statusline) for [Claude Code](https://docs.anthropic.com/en/docs/claude-code).

**This is a personal tool.** It's published as a starting point for forking and customizing, not as a general-purpose library. The layout is hardcoded to my preferences — fork it and make it yours.

![example output](docs/example.svg)

## Why speed matters

Claude Code's status line command [runs on every prompt refresh](https://code.claude.com/docs/en/statusline) (300ms debounce). A typical shell script spawns multiple processes per invocation (bash, jq, git), adding up to ~170ms. This Rust binary uses native libraries (libgit2, serde) to do the same work in ~13ms — about 10x faster.

It's also kinder to your battery. Over a full day of heavy coding, those saved milliseconds add up to less CPU time and less heat. Napkin math puts the annual energy savings at a few cents at Hydro-Québec rates. The API call that just answered your prompt probably used more electricity, but at least *your* fan stays quiet.

## Fork and customize

The layout is hardcoded in `src/main.rs` (~80 lines). There's no config file by design — editing source and running `cargo build --release` is faster than parsing config on every invocation.

Claude Code sends a [rich JSON payload](https://code.claude.com/docs/en/statusline#available-data) on stdin with every refresh. This project currently uses only a subset:

| Used | Available but unused |
|------|---------------------|
| `workspace.current_dir` | `model.id` |
| `model.display_name` | `cost.total_lines_added/removed` |
| `cost.total_cost_usd` | `vim.mode`, `session_id`, `worktree.*` |
| `context_window.total_input_tokens` | |
| `context_window.total_output_tokens` | |
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
| Model | muted green `#7a9e56` | `model.display_name` |
| Path | muted cyan `#5a9ea0` | Last 2 of `workspace.current_dir` |
| Git | muted purple `#7a6db0` | `git2` branch + dirty |
| Context | muted yellow `#b09a42` | `used_percentage`/`context_window_size` |
| Tokens+Cost | light gray | `context_window` total + `cost.total_cost_usd` |
| Separators | dark gray | `\x1b[90m` |

## Benchmarking

```bash
mise run bench
```

Compares the Rust binary against a POSIX shell equivalent and other status line tools using [hyperfine](https://github.com/sharkdp/hyperfine).

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `ccline` | 12.8 ± 1.5 | 10.9 | 19.5 | 1.00 |
| `ccline.sh (via bash)` | 169.4 ± 2.4 | 166.2 | 173.8 | 13.23 ± 1.54 |
| `ccline.sh (via sh)` | 178.3 ± 2.9 | 174.5 | 185.6 | 13.93 ± 1.62 |
| `ccline.sh (via zsh)` | 184.8 ± 3.2 | 179.9 | 190.4 | 14.43 ± 1.68 |
| `starship-claude (defaults)` | 200.4 ± 2.5 | 197.1 | 205.1 | 15.66 ± 1.82 |
| `ccstatusline (defaults)` | 282.7 ± 4.3 | 276.1 | 289.6 | 22.08 ± 2.57 |
| `claude-powerline (defaults)` | 342.2 ± 20.2 | 309.1 | 367.8 | 26.74 ± 3.46 |

## See also

- [Owloops/claude-powerline](https://github.com/Owloops/claude-powerline) — a feature-rich, configurable status line plugin
- [sirmalloc/ccstatusline](https://github.com/sirmalloc/ccstatusline) — a Node.js status line with built-in themes
- [martinemde/starship-claude](https://github.com/martinemde/starship-claude) — bridges Claude Code data into a Starship prompt

## License

MIT

---

Made with ❤️ in Montreal
