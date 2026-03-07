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
JSON on stdin: `{"workspace":{"current_dir":"...","project_dir":"...","added_dirs":[]}}`

## Output
ANSI line: `user@host \033[34m/path\033[0m \033[90mbranch\033[0m\033[36m*\033[0m`

## Benchmarking
`bench.sh` compares the Rust binary against an equivalent bash script using hyperfine.
When changing the output format of `ccline`, always update `bench.sh` to match:
- The sample JSON input must include all fields the binary reads
- The bash comparison script must produce identical output to the Rust binary
