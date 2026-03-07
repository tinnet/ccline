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
    if io::stdin().read_to_string(&mut buf).is_err() {
        return;
    }
    let input: Input = match serde_json::from_str(&buf) {
        Ok(v) => v,
        Err(_) => return,
    };
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
