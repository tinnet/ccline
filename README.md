# ccline

A fast [status line](https://code.claude.com/docs/en/statusline) for [Claude Code](https://docs.anthropic.com/en/docs/claude-code), written in Rust.

**This is a personal tool.** It's published as a starting point for forking and customizing, not as a general-purpose library. The layout is hardcoded to my preferences — fork it and make it yours.

```
user@hostname ~/projects/myapp main*
```

## Why Rust

Claude Code's status line command [runs on every prompt refresh](https://code.claude.com/docs/en/statusline) (300ms debounce). A typical bash script spawns multiple processes per invocation (bash, jq, git), adding up to ~280ms. This Rust binary uses native libraries (libgit2, serde) to do the same work in ~1.4ms — about 200x faster.

## Fork and customize

The layout is hardcoded in `src/main.rs` (~50 lines). There's no config file by design — editing source and running `cargo build --release` is faster than parsing config on every invocation.

Claude Code sends a [rich JSON payload](https://code.claude.com/docs/en/statusline#available-data) on stdin with every refresh. This project currently uses only a subset:

| Used | Available but unused |
|------|---------------------|
| `workspace.current_dir` | `model.id`, `model.display_name` |
| (git via libgit2) | `cost.total_cost_usd` |
| | `context_window.used_percentage` |
| | `context_window.context_window_size` |
| | `cost.total_lines_added/removed` |
| | `vim.mode`, `session_id`, `worktree.*` |

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

| Segment | Color | Source |
|---------|-------|--------|
| `user@host` | default | `$USER` + `gethostname` |
| `/path` | blue | `workspace.current_dir` from stdin JSON |
| `branch` | gray | `git2` (libgit2) |
| `*` | cyan | dirty working tree (unstaged, staged, or untracked) |

## Benchmarking

```bash
mise run bench
```

Compares the Rust binary against an equivalent bash script using [hyperfine](https://github.com/sharkdp/hyperfine).

## License

MIT
