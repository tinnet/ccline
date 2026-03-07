# cld-sts-line: Rust Status Line for Claude Code

## Problem

The Claude Code TUI status line is currently a bash script. It works but is slow due to process spawning (bash, jq, git). A compiled Rust binary can do the same work in under 1ms.

## Approach

Single Rust binary with hardcoded layout. No config file, no CLI args. Personal tool — recompile to change layout.

## Input

JSON on stdin from Claude Code:

```json
{
  "workspace": {
    "current_dir": "/path/to/project",
    "project_dir": "/path/to/project",
    "added_dirs": []
  }
}
```

## Output

Single ANSI-colored line to stdout:

```
user@host cwd branch*
```

- `user@host` — default color
- `cwd` — blue (`\033[34m`)
- `branch` — gray (`\033[90m`)
- `*` (dirty) — cyan (`\033[36m`)

Dirty means any of: unstaged changes, staged changes, untracked files.

## Dependencies

| Crate | Purpose |
|-------|---------|
| `serde` + `serde_json` | Parse stdin JSON |
| `git2` | Branch name + dirty state |
| `gethostname` | Hostname without shelling out |
| `assert_cmd` + `predicates` (dev) | Integration tests |

## Project Structure

Based on [tinnet/rust-cli-template](https://github.com/tinnet/rust-cli-template) with `clap`, `shadow-rs`, `build.rs`, and `dist-workspace.toml` removed.

```
cld-sts-line/
├── .claude/settings.local.json   # Claude Code hooks (auto-bench)
├── CLAUDE.md
├── Cargo.toml
├── bench.sh                      # hyperfine benchmarks
├── hk.pkl                        # Pre-commit hooks
├── mise.toml                     # Build/test/lint/bench tasks
├── src/main.rs                   # Single file, ~80 lines
└── tests/cli.rs                  # Integration tests
```

## Benchmarking

- `bench.sh` uses hyperfine to compare the Rust binary against the current bash script
- `mise run bench` builds release and runs benchmarks
- Claude Code hook auto-runs `./bench.sh` after `cargo build --release`

## What We Removed from Template

- `clap` — no CLI args needed
- `shadow-rs` + `build.rs` — no version embedding
- `dist-workspace.toml` + `cargo-dist` — no release distribution

## What We Kept from Template

- `mise.toml` — task runner with added `bench` task
- `hk.pkl` — pre-commit formatting + secret scanning
- `assert_cmd` — integration tests adapted for stdin/stdout
- `.gitignore`, CI, `CLAUDE.md` structure
