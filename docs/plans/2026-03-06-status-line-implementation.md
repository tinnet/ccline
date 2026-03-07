# cld-sts-line Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Rust CLI that reads Claude Code's status line JSON from stdin and outputs a formatted ANSI status line with user, host, cwd, git branch, and dirty state.

**Architecture:** Single-file Rust binary. Reads JSON from stdin, gets username/hostname from environment/OS, opens git repo via libgit2, prints one ANSI-colored line to stdout. No CLI args, no config.

**Tech Stack:** Rust 2024 edition, serde/serde_json, git2, gethostname, assert_cmd (dev), hyperfine (bench)

---

### Task 1: Scaffold project from template

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `mise.toml`
- Create: `hk.pkl`
- Create: `.gitignore`
- Create: `CLAUDE.md`

**Step 1: Initialize Cargo project**

Run: `cargo init --name cld-sts-line .`

This will create `Cargo.toml` and `src/main.rs`. We'll overwrite both.

**Step 2: Write Cargo.toml**

```toml
[package]
name = "cld-sts-line"
version = "0.1.0"
edition = "2024"

[dependencies]
gethostname = "1"
git2 = "0.20"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[dev-dependencies]
assert_cmd = "2.1"
predicates = "3.1"

[profile.release]
lto = "thin"
```

**Step 3: Write starter main.rs**

```rust
fn main() {
    println!("hello");
}
```

**Step 4: Write .gitignore**

```
/target
```

**Step 5: Write CLAUDE.md**

```markdown
# CLAUDE.md

## Project Overview
Rust CLI that serves as Claude Code's status line. Reads JSON from stdin, outputs ANSI-formatted status line.

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
```

**Step 6: Write mise.toml**

```toml
[tools]
rust = "latest"
"github:betterleaks/betterleaks" = "latest"
hk = "latest"

[tasks.build]
run = "cargo build"
description = "Compile"
depends = ["test"]

[tasks.test]
run = "cargo test"
description = "Run tests"

[tasks.format]
alias = "fmt"
run = "cargo fmt"
description = "Auto-format code"

[tasks.lint]
run = "cargo clippy -- -D warnings && cargo fmt --check"
description = "Check formatting + lint"

[tasks.bench]
run = "cargo build --release && ./bench.sh"
description = "Build release and benchmark"

[tasks.scan-secrets]
run = "betterleaks git --pre-commit"
description = "Scan for secrets in staged changes"
```

**Step 7: Write hk.pkl**

```pkl
amends "package://github.com/jdx/hk/releases/download/v1.37.0/hk@1.37.0#/Config.pkl"
import "package://github.com/jdx/hk/releases/download/v1.37.0/hk@1.37.0#/Builtins.pkl"

local linters = new Mapping<String, Step> {
    ["format"] {
        workspace_indicator = "Cargo.toml"
        check = "mise run format"
    }
    ["scan"] {
        workspace_indicator = "Cargo.toml"
        check = "mise run scan-secrets"
    }
}

hooks {
    ["pre-commit"] {
        fix = true
        stash = "git"
        steps = linters
    }
}
```

**Step 8: Verify it compiles**

Run: `cargo build`
Expected: compiles successfully, prints "hello" when run

**Step 9: Commit**

```bash
git add Cargo.toml Cargo.lock src/main.rs .gitignore CLAUDE.md mise.toml hk.pkl
git commit -m "feat: scaffold project from rust-cli-template"
```

---

### Task 2: Parse stdin JSON

**Files:**
- Create: `tests/cli.rs`
- Modify: `src/main.rs`

**Step 1: Write the failing test**

Create `tests/cli.rs`:

```rust
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn parses_workspace_cwd() {
    let input = r#"{"workspace":{"current_dir":"/tmp/test","project_dir":"/tmp/test","added_dirs":[]}}"#;
    let mut cmd = Command::cargo_bin("cld-sts-line").unwrap();
    cmd.write_stdin(input);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("/tmp/test"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test cli parses_workspace_cwd -- --nocapture`
Expected: FAIL — main.rs just prints "hello", doesn't contain "/tmp/test"

**Step 3: Write minimal implementation**

Replace `src/main.rs`:

```rust
use serde::Deserialize;
use std::io::{self, Read};

#[derive(Deserialize)]
struct Input {
    workspace: Workspace,
}

#[derive(Deserialize)]
struct Workspace {
    current_dir: String,
}

fn main() {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf).unwrap();
    let input: Input = serde_json::from_str(&buf).unwrap();
    print!("{}", input.workspace.current_dir);
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test cli parses_workspace_cwd -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/main.rs tests/cli.rs
git commit -m "feat: parse workspace JSON from stdin"
```

---

### Task 3: Add user@host prefix

**Files:**
- Modify: `tests/cli.rs`
- Modify: `src/main.rs`

**Step 1: Write the failing test**

Add to `tests/cli.rs`:

```rust
#[test]
fn includes_user_and_host() {
    let input = r#"{"workspace":{"current_dir":"/tmp/test","project_dir":"/tmp/test","added_dirs":[]}}"#;
    let user = std::env::var("USER").unwrap();
    let mut cmd = Command::cargo_bin("cld-sts-line").unwrap();
    cmd.write_stdin(input);
    cmd.assert()
        .success()
        .stdout(predicate::str::starts_with(&format!("{}@", user)));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test cli includes_user_and_host -- --nocapture`
Expected: FAIL — output starts with "/tmp/test", not "user@host"

**Step 3: Implement user@host**

Update `main()` in `src/main.rs`:

```rust
fn main() {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf).unwrap();
    let input: Input = serde_json::from_str(&buf).unwrap();

    let user = std::env::var("USER").unwrap_or_else(|_| "?".into());
    let host = gethostname::gethostname();
    let host = host.to_string_lossy();
    // Strip .local suffix from macOS hostnames
    let host = host.strip_suffix(".local").unwrap_or(&host);

    print!("{}@{} {}", user, host, input.workspace.current_dir);
}
```

Add `use` at top: already have `gethostname` in Cargo.toml.

**Step 4: Run test to verify it passes**

Run: `cargo test --test cli -- --nocapture`
Expected: both tests PASS

**Step 5: Commit**

```bash
git add src/main.rs tests/cli.rs
git commit -m "feat: add user@host prefix"
```

---

### Task 4: Add ANSI color to cwd

**Files:**
- Modify: `tests/cli.rs`
- Modify: `src/main.rs`

**Step 1: Write the failing test**

Add to `tests/cli.rs`:

```rust
#[test]
fn cwd_is_blue() {
    let input = r#"{"workspace":{"current_dir":"/tmp/test","project_dir":"/tmp/test","added_dirs":[]}}"#;
    let mut cmd = Command::cargo_bin("cld-sts-line").unwrap();
    cmd.write_stdin(input);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\x1b[34m/tmp/test\x1b[0m"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test cli cwd_is_blue -- --nocapture`
Expected: FAIL — no ANSI codes in output yet

**Step 3: Add ANSI codes**

Update the `print!` in `main()`:

```rust
    print!(
        "{}@{} \x1b[34m{}\x1b[0m",
        user, host, input.workspace.current_dir
    );
```

**Step 4: Run tests**

Run: `cargo test --test cli -- --nocapture`
Expected: all tests PASS

**Step 5: Commit**

```bash
git add src/main.rs tests/cli.rs
git commit -m "feat: add blue ANSI color to cwd"
```

---

### Task 5: Add git branch and dirty state

**Files:**
- Modify: `tests/cli.rs`
- Modify: `src/main.rs`

**Step 1: Write the failing test for git branch**

Add to `tests/cli.rs`:

```rust
#[test]
fn shows_git_branch() {
    // Use the project's own repo as test input — we know it's a git repo
    let cwd = std::env::current_dir().unwrap();
    let input = format!(
        r#"{{"workspace":{{"current_dir":"{}","project_dir":"{}","added_dirs":[]}}}}"#,
        cwd.display(),
        cwd.display()
    );
    let mut cmd = Command::cargo_bin("cld-sts-line").unwrap();
    cmd.write_stdin(input.as_str());
    // Should contain the gray ANSI code for a branch name
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\x1b[90m"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test cli shows_git_branch -- --nocapture`
Expected: FAIL — no git info in output

**Step 3: Write a test for non-git directory**

Add to `tests/cli.rs`:

```rust
#[test]
fn works_without_git() {
    let input = r#"{"workspace":{"current_dir":"/tmp","project_dir":"/tmp","added_dirs":[]}}"#;
    let mut cmd = Command::cargo_bin("cld-sts-line").unwrap();
    cmd.write_stdin(input);
    cmd.assert()
        .success()
        // Should NOT contain git ANSI codes
        .stdout(predicate::str::contains("\x1b[90m").not());
}
```

**Step 4: Implement git branch + dirty state**

Update `src/main.rs`:

```rust
use git2::Repository;
use serde::Deserialize;
use std::io::{self, Read};

#[derive(Deserialize)]
struct Input {
    workspace: Workspace,
}

#[derive(Deserialize)]
struct Workspace {
    current_dir: String,
}

fn git_info(path: &str) -> Option<String> {
    let repo = Repository::open(path).ok()?;
    let head = repo.head().ok()?;
    let branch = head.shorthand()?.to_string();

    let dirty = repo
        .statuses(Some(
            git2::StatusOptions::new()
                .include_untracked(true)
                .exclude_submodules(true),
        ))
        .ok()
        .map_or(false, |s| !s.is_empty());

    let dirty_marker = if dirty { "\x1b[36m*\x1b[0m" } else { "" };
    Some(format!(" \x1b[90m{}\x1b[0m{}", branch, dirty_marker))
}

fn main() {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf).unwrap();
    let input: Input = serde_json::from_str(&buf).unwrap();

    let user = std::env::var("USER").unwrap_or_else(|_| "?".into());
    let host = gethostname::gethostname();
    let host = host.to_string_lossy();
    let host = host.strip_suffix(".local").unwrap_or(&host);

    let git = git_info(&input.workspace.current_dir).unwrap_or_default();

    print!(
        "{}@{} \x1b[34m{}\x1b[0m{}",
        user, host, input.workspace.current_dir, git
    );
}
```

**Step 5: Run all tests**

Run: `cargo test -- --nocapture`
Expected: all tests PASS

**Step 6: Commit**

```bash
git add src/main.rs tests/cli.rs
git commit -m "feat: add git branch and dirty state indicator"
```

---

### Task 6: Benchmark setup

**Files:**
- Create: `bench.sh`
- Modify: `mise.toml` (already done in Task 1, verify)
- Modify: `.claude/settings.local.json`

**Step 1: Write bench.sh**

```bash
#!/usr/bin/env bash
set -euo pipefail

SAMPLE_JSON='{"workspace":{"current_dir":"'"$(pwd)"'","project_dir":"'"$(pwd)"'","added_dirs":[]}}'

BASH_CMD='input=$(cat); cwd=$(echo "$input" | jq -r '"'"'.workspace.current_dir'"'"'); user=$(whoami); host=$(hostname -s); git_info=""; if git -C "$cwd" rev-parse --git-dir >/dev/null 2>&1; then branch=$(git -C "$cwd" --no-optional-locks branch --show-current 2>/dev/null || echo ""); if [ -n "$branch" ]; then if ! git -C "$cwd" --no-optional-locks diff --quiet 2>/dev/null || ! git -C "$cwd" --no-optional-locks diff --cached --quiet 2>/dev/null || [ -n "$(git -C "$cwd" --no-optional-locks ls-files --others --exclude-standard 2>/dev/null)" ]; then status="*"; else status=""; fi; git_info=" $(printf '"'"'\033[90m'"'"')${branch}$(printf '"'"'\033[0m'"'"')$(printf '"'"'\033[36m'"'"')${status}$(printf '"'"'\033[0m'"'"')"; fi; fi; printf "%s@%s $(printf '"'"'\033[34m'"'"')%s$(printf '"'"'\033[0m'"'"')%s" "$user" "$host" "$cwd" "$git_info"'

echo "Benchmarking with input: $SAMPLE_JSON"
echo ""

hyperfine \
    --warmup 3 \
    --runs 50 \
    --input <(echo "$SAMPLE_JSON") \
    --command-name "rust" "./target/release/cld-sts-line" \
    --command-name "bash" "bash -c '$BASH_CMD'"
```

**Step 2: Make it executable**

Run: `chmod +x bench.sh`

**Step 3: Add Claude Code hook for auto-benchmarking**

Update `.claude/settings.local.json`:

```json
{
  "enabledPlugins": {
    "superpowers@claude-plugins-official": true
  },
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "if echo \"$CLAUDE_TOOL_INPUT\" | grep -q 'cargo build.*--release'; then ./bench.sh; fi"
          }
        ]
      }
    ]
  }
}
```

**Step 4: Build release and run first benchmark**

Run: `cargo build --release && ./bench.sh`
Expected: hyperfine output showing both rust and bash timings

**Step 5: Commit**

```bash
git add bench.sh .claude/settings.local.json
git commit -m "feat: add hyperfine benchmarks and auto-bench hook"
```

---

### Task 7: Wire up in Claude Code settings

**This task is manual — document the steps for the user.**

After the binary is built and benchmarked, update `~/.claude/settings.json` to use the Rust binary:

```json
"statusLine": {
  "type": "command",
  "command": "/path/to/cld-sts-line/target/release/cld-sts-line"
}
```

Or install it system-wide first:

```bash
cargo install --path .
```

Then:

```json
"statusLine": {
  "type": "command",
  "command": "cld-sts-line"
}
```

Verify it works by restarting Claude Code.

---
