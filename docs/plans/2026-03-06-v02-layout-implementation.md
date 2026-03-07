# v0.2 Layout Redesign Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Redesign ccline's output to show model, short path, git, tokens, and cost in a pipe-separated layout with muted Monokai Pro colors.

**Architecture:** Expand serde structs for new JSON fields, add helper functions for token/path formatting, replace the print statement, update all tests and bench.sh.

**Tech Stack:** Rust, serde/serde_json, git2, gethostname (remove after Task 2)

---

### Task 1: Add helper functions with unit tests

**Files:**
- Modify: `src/main.rs`

**Step 1: Write failing unit tests for `human_tokens` and `short_path`**

Add to the bottom of `src/main.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_tokens_small() {
        assert_eq!(human_tokens(847), "847 tks");
    }

    #[test]
    fn test_human_tokens_low_k() {
        assert_eq!(human_tokens(1234), "1.2k tks");
    }

    #[test]
    fn test_human_tokens_mid_k() {
        assert_eq!(human_tokens(42000), "42k tks");
    }

    #[test]
    fn test_human_tokens_millions() {
        assert_eq!(human_tokens(1_523_400), "1.5M tks");
    }

    #[test]
    fn test_human_tokens_zero() {
        assert_eq!(human_tokens(0), "0 tks");
    }

    #[test]
    fn test_short_path_two_components() {
        assert_eq!(short_path("/Users/selkie/src/github.com/tinnet/ccline"), "tinnet/ccline");
    }

    #[test]
    fn test_short_path_one_component() {
        assert_eq!(short_path("/tmp"), "tmp");
    }

    #[test]
    fn test_short_path_root() {
        assert_eq!(short_path("/"), "/");
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib`
Expected: FAIL — `human_tokens` and `short_path` don't exist yet

**Step 3: Implement the helper functions**

Add above the `fn main()` in `src/main.rs`:

```rust
fn human_tokens(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M tks", n as f64 / 1_000_000.0)
    } else if n >= 10_000 {
        format!("{}k tks", n / 1000)
    } else if n >= 1_000 {
        format!("{:.1}k tks", n as f64 / 1000.0)
    } else {
        format!("{} tks", n)
    }
}

fn short_path(path: &str) -> String {
    let components: Vec<&str> = path.rsplitn(3, '/').collect();
    match components.len() {
        0 => path.to_string(),
        1 => components[0].to_string(),
        2 => {
            if components[1].is_empty() {
                // Path like "/foo" -> just "foo"
                components[0].to_string()
            } else {
                format!("{}/{}", components[1], components[0])
            }
        }
        _ => {
            if components[0].is_empty() && components[1].is_empty() {
                // Root path "/"
                "/".to_string()
            } else {
                format!("{}/{}", components[1], components[0])
            }
        }
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --lib`
Expected: PASS — all 8 unit tests green

**Step 5: Commit**

```bash
git add src/main.rs
git commit -m "feat: add human_tokens and short_path helper functions"
```

---

### Task 2: Expand serde structs and rewrite output

**Files:**
- Modify: `src/main.rs`

**Step 1: Update the serde structs**

Replace the existing `Input` and `Workspace` structs at the top of `src/main.rs` with:

```rust
#[derive(Deserialize)]
struct Input {
    workspace: Workspace,
    model: Option<Model>,
    cost: Option<Cost>,
    context_window: Option<ContextWindow>,
}

#[derive(Deserialize)]
struct Workspace {
    current_dir: String,
}

#[derive(Deserialize)]
struct Model {
    display_name: String,
}

#[derive(Deserialize)]
struct Cost {
    total_cost_usd: f64,
}

#[derive(Deserialize)]
struct ContextWindow {
    total_input_tokens: u64,
    total_output_tokens: u64,
}
```

**Step 2: Define color constants**

Add after the struct definitions:

```rust
// Monokai Pro palette at ~60% brightness
const GREEN: &str = "\x1b[38;2;122;158;86m";
const CYAN: &str = "\x1b[38;2;90;158;160m";
const PURPLE: &str = "\x1b[38;2;122;109;176m";
const YELLOW: &str = "\x1b[38;2;176;154;66m";
const PINK: &str = "\x1b[38;2;176;74;96m";
const GRAY: &str = "\x1b[90m";
const RESET: &str = "\x1b[0m";
```

**Step 3: Update `git_info` to use new color constants**

Replace the `git_info` function:

```rust
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

    let dirty_marker = if dirty { "*" } else { "" };
    Some(format!("{PURPLE}{}{dirty_marker}{RESET}", branch))
}
```

**Step 4: Rewrite `main()` with the new layout**

Replace the entire `main()` function:

```rust
fn main() {
    let mut buf = String::new();
    if io::stdin().read_to_string(&mut buf).is_err() {
        return;
    }
    let input: Input = match serde_json::from_str(&buf) {
        Ok(v) => v,
        Err(_) => return,
    };

    let sep = format!(" {GRAY}|{RESET} ");
    let mut segments: Vec<String> = Vec::new();

    // Model name
    if let Some(ref model) = input.model {
        segments.push(format!("{GREEN}{}{RESET}", model.display_name));
    }

    // Short path
    segments.push(format!("{CYAN}{}{RESET}", short_path(&input.workspace.current_dir)));

    // Git branch + dirty
    if let Some(git) = git_info(&input.workspace.current_dir) {
        segments.push(git);
    }

    // Token count
    if let Some(ref ctx) = input.context_window {
        let total = ctx.total_input_tokens + ctx.total_output_tokens;
        segments.push(format!("{YELLOW}{}{RESET}", human_tokens(total)));
    }

    // Session cost
    if let Some(ref cost) = input.cost {
        segments.push(format!("{PINK}${:.2}{RESET}", cost.total_cost_usd));
    }

    print!("{}", segments.join(&sep));
}
```

**Step 5: Remove gethostname dependency**

Since we no longer show `user@host`, remove from `Cargo.toml`:

```toml
# Remove this line:
gethostname = "1"
```

And remove `use gethostname` if present (it's used inline so just removing the dep suffices).

**Step 6: Verify it compiles**

Run: `cargo build`
Expected: compiles successfully

**Step 7: Commit**

```bash
git add src/main.rs Cargo.toml Cargo.lock
git commit -m "feat: new v0.2 layout with model, tokens, cost, and Monokai colors"
```

---

### Task 3: Update integration tests

**Files:**
- Modify: `tests/cli.rs`

**Step 1: Replace all integration tests**

Replace the entire contents of `tests/cli.rs`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

fn full_json() -> String {
    let cwd = std::env::current_dir().unwrap();
    format!(
        r#"{{
            "workspace":{{"current_dir":"{}","project_dir":"{}","added_dirs":[]}},
            "model":{{"id":"claude-opus-4-6","display_name":"Opus"}},
            "cost":{{"total_cost_usd":0.12}},
            "context_window":{{"total_input_tokens":30000,"total_output_tokens":12000}}
        }}"#,
        cwd.display(),
        cwd.display()
    )
}

fn minimal_json() -> &'static str {
    r#"{"workspace":{"current_dir":"/tmp/foo/bar"}}"#
}

#[test]
fn shows_model_name() {
    let mut cmd = Command::cargo_bin("ccline").unwrap();
    cmd.write_stdin(full_json());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Opus"));
}

#[test]
fn shows_short_path() {
    let mut cmd = Command::cargo_bin("ccline").unwrap();
    cmd.write_stdin(minimal_json());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("foo/bar"));
}

#[test]
fn shows_git_branch_in_repo() {
    let mut cmd = Command::cargo_bin("ccline").unwrap();
    cmd.write_stdin(full_json());
    // Purple ANSI code for git branch
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\x1b[38;2;122;109;176m"));
}

#[test]
fn no_git_outside_repo() {
    let mut cmd = Command::cargo_bin("ccline").unwrap();
    cmd.write_stdin(minimal_json());
    // Should not contain purple ANSI code
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\x1b[38;2;122;109;176m").not());
}

#[test]
fn shows_token_count() {
    let mut cmd = Command::cargo_bin("ccline").unwrap();
    cmd.write_stdin(full_json());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("42k tks"));
}

#[test]
fn shows_cost() {
    let mut cmd = Command::cargo_bin("ccline").unwrap();
    cmd.write_stdin(full_json());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("$0.12"));
}

#[test]
fn shows_pipe_separators() {
    let mut cmd = Command::cargo_bin("ccline").unwrap();
    cmd.write_stdin(full_json());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("|"));
}

#[test]
fn works_with_minimal_json() {
    // Only workspace, no model/cost/context — should not crash
    let mut cmd = Command::cargo_bin("ccline").unwrap();
    cmd.write_stdin(minimal_json());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("foo/bar"));
}

#[test]
fn no_user_host() {
    // Ensure the old user@host format is gone
    let user = std::env::var("USER").unwrap_or_default();
    let mut cmd = Command::cargo_bin("ccline").unwrap();
    cmd.write_stdin(full_json());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(&format!("{}@", user)).not());
}
```

**Step 2: Run all tests**

Run: `cargo test`
Expected: all unit tests (8) and integration tests (9) pass

**Step 3: Commit**

```bash
git add tests/cli.rs
git commit -m "test: update integration tests for v0.2 layout"
```

---

### Task 4: Update bench.sh

**Files:**
- Modify: `bench.sh`

**Step 1: Rewrite bench.sh**

Replace the entire contents of `bench.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

CWD="$(pwd)"
LAST_TWO=$(echo "$CWD" | rev | cut -d/ -f1-2 | rev)

SAMPLE_JSON='{"workspace":{"current_dir":"'"$CWD"'","project_dir":"'"$CWD"'","added_dirs":[]},"model":{"id":"claude-opus-4-6","display_name":"Opus"},"cost":{"total_cost_usd":0.12},"context_window":{"total_input_tokens":30000,"total_output_tokens":12000}}'

# Bash equivalent of the Rust binary's output
BASH_CMD='input=$(cat)
cwd=$(echo "$input" | jq -r ".workspace.current_dir")
model=$(echo "$input" | jq -r ".model.display_name")
cost=$(echo "$input" | jq -r ".cost.total_cost_usd")
in_tks=$(echo "$input" | jq -r ".context_window.total_input_tokens")
out_tks=$(echo "$input" | jq -r ".context_window.total_output_tokens")
total_tks=$((in_tks + out_tks))
last_two=$(echo "$cwd" | rev | cut -d/ -f1-2 | rev)

# Token formatting
if [ "$total_tks" -ge 1000000 ]; then
  tks=$(awk "BEGIN{printf \"%.1fM tks\", $total_tks/1000000}")
elif [ "$total_tks" -ge 10000 ]; then
  tks="$((total_tks / 1000))k tks"
elif [ "$total_tks" -ge 1000 ]; then
  tks=$(awk "BEGIN{printf \"%.1fk tks\", $total_tks/1000}")
else
  tks="${total_tks} tks"
fi

cost_fmt=$(printf "$%.2f" "$cost")

# Colors (Monokai Pro ~60%)
GREEN="\033[38;2;122;158;86m"
CYAN="\033[38;2;90;158;160m"
PURPLE="\033[38;2;122;109;176m"
YELLOW="\033[38;2;176;154;66m"
PINK="\033[38;2;176;74;96m"
GRAY="\033[90m"
RST="\033[0m"

SEP=" ${GRAY}|${RST} "

git_info=""
if git -C "$cwd" rev-parse --git-dir >/dev/null 2>&1; then
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

printf "${GREEN}${model}${RST}${SEP}${CYAN}${last_two}${RST}${git_info}${SEP}${YELLOW}${tks}${RST}${SEP}${PINK}${cost_fmt}${RST}"'

echo "Benchmarking with input:"
echo "$SAMPLE_JSON" | jq .
echo ""

hyperfine \
    --warmup 3 \
    --runs 50 \
    --input <(echo "$SAMPLE_JSON") \
    --command-name "rust" "./target/release/ccline" \
    --command-name "bash" "bash -c '$BASH_CMD'"
```

**Step 2: Verify benchmark runs**

Run: `cargo build --release && ./bench.sh`
Expected: hyperfine output showing both timings, Rust significantly faster

**Step 3: Commit**

```bash
git add bench.sh
git commit -m "bench: update bench.sh for v0.2 layout"
```

---

### Task 5: Update README

**Files:**
- Modify: `README.md`

**Step 1: Update the README**

Update the example output, the layout table, and the "Used vs Available" table. Add "See also" section. Key changes:

- Example output: `Opus | tinnet/ccline | main* | 42k tks | $0.12`
- Layout table: new segments with Monokai colors
- Used table: model, tokens, cost now in "Used" column
- Add "See also" section at the bottom with link to claude-powerline

**Step 2: Commit**

```bash
git add README.md
git commit -m "docs: update README for v0.2 layout"
```

---

### Task 6: Update CLAUDE.md

**Files:**
- Modify: `CLAUDE.md`

**Step 1: Update the Output section**

Update the output format description and the "key fields" list to reflect that we now use model, cost, and context_window.

**Step 2: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: update CLAUDE.md for v0.2 layout"
```
