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
    if path == "/" {
        return "/".to_string();
    }
    let components: Vec<&str> = path.rsplitn(3, '/').collect();
    match components.len() {
        0 => path.to_string(),
        1 => components[0].to_string(),
        2 => {
            if components[1].is_empty() {
                components[0].to_string()
            } else {
                format!("{}/{}", components[1], components[0])
            }
        }
        _ => {
            format!("{}/{}", components[1], components[0])
        }
    }
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
