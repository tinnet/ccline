# ccline

A fast [status line](https://code.claude.com/docs/en/statusline) for [Claude Code](https://docs.anthropic.com/en/docs/claude-code), written in Rust.

**This is a personal tool.** It's published as a starting point for forking and customizing, not as a general-purpose library. The layout is hardcoded to my preferences — fork it and make it yours.

<p>
  <span style="color:#7a9e56">Opus</span>
  <span style="color:#555"> | </span>
  <span style="color:#5a9ea0">tinnet/ccline</span>
  <span style="color:#555"> | </span>
  <span style="color:#7a6db0">main*</span>
  <span style="color:#555"> | </span>
  <span style="color:#b09a42">42k tks</span>
  <span style="color:#555"> | </span>
  <span style="color:#b04a60">$0.12</span>
</p>

## Why Rust

Claude Code's status line command [runs on every prompt refresh](https://code.claude.com/docs/en/statusline) (300ms debounce). A typical bash script spawns multiple processes per invocation (bash, jq, git), adding up to ~280ms. This Rust binary uses native libraries (libgit2, serde) to do the same work in ~1.4ms — about 200x faster.

## Fork and customize

The layout is hardcoded in `src/main.rs` (~80 lines). There's no config file by design — editing source and running `cargo build --release` is faster than parsing config on every invocation.

Claude Code sends a [rich JSON payload](https://code.claude.com/docs/en/statusline#available-data) on stdin with every refresh. This project currently uses only a subset:

| Used | Available but unused |
|------|---------------------|
| `workspace.current_dir` | `model.id` |
| `model.display_name` | `cost.total_lines_added/removed` |
| `cost.total_cost_usd` | `context_window.used_percentage` |
| `context_window.total_input_tokens` | `context_window.context_window_size` |
| `context_window.total_output_tokens` | `vim.mode`, `session_id`, `worktree.*` |
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
| Tokens | muted yellow `#b09a42` | `context_window` total |
| Cost | muted pink `#b04a60` | `cost.total_cost_usd` |
| Separators | dark gray | `\x1b[90m` |

## Benchmarking

```bash
mise run bench
```

Compares the Rust binary against an equivalent bash script using [hyperfine](https://github.com/sharkdp/hyperfine).

## See also

- [claude-powerline](https://github.com/Owloops/claude-powerline) — a feature-rich, configurable status line plugin

## License

MIT
