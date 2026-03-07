# CLAUDE.md

## Project Overview
`ccline` — Rust CLI that serves as Claude Code's status line. Reads JSON from stdin, outputs ANSI-formatted status line.

## Build & Test
- `cargo build` / `cargo build --release`
- `cargo test`
- `mise run bench` — benchmark against bash baseline

## Architecture
Single file: `src/main.rs`. No CLI args, no config. Hardcoded layout.

## Input
JSON on stdin from Claude Code. Full schema: https://code.claude.com/docs/en/statusline#available-data

Key fields used: `workspace.current_dir`, `model.display_name`, `cost.total_cost_usd`, `context_window.total_input_tokens`, `context_window.total_output_tokens`

## Output
Pipe-separated ANSI line: `Model | path | branch* | tokens | $cost`

`docs/example.svg` shows the colored output in the README. Update it whenever the layout changes.

## Benchmarking
`bench.sh` compares the Rust binary against an equivalent bash script using hyperfine.
When changing the output format of `ccline`, always update `bench.sh` to match:
- The sample JSON input must include all fields the binary reads
- The bash comparison script must produce identical output to the Rust binary

## Testing
Integration tests in `tests/cli.rs` use `assert_cmd` to invoke the binary and pipe JSON on stdin. Tests assert on stdout content including ANSI escape codes.

## Releasing
Releases are managed by cargo-dist. To release:
1. Bump version in `Cargo.toml`
2. `git commit && git tag v<version> && git push && git push origin v<version>`
3. GitHub Actions builds binaries for macOS, Linux, and Windows
4. Install via `mise use -g github:tinnet/ccline`
