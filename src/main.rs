use git2::Repository;
use serde::Deserialize;
use std::io::{self, Read};

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
    context_window_size: Option<u64>,
}

// Monokai Pro palette at ~60% brightness
const GREEN: &str = "\x1b[38;2;122;158;86m";
const CYAN: &str = "\x1b[38;2;90;158;160m";
const PURPLE: &str = "\x1b[38;2;122;109;176m";
const YELLOW: &str = "\x1b[38;2;176;154;66m";
const PINK: &str = "\x1b[38;2;176;74;96m";
const GRAY: &str = "\x1b[90m";
const RESET: &str = "\x1b[0m";

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

fn human_tokens(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 10_000 {
        format!("{}k", n / 1000)
    } else if n >= 1_000 {
        format!("{:.1}k", n as f64 / 1000.0)
    } else {
        format!("{}", n)
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
        let token_str = match ctx.context_window_size {
            Some(window) => format!("{}/{} tks", human_tokens(total), human_tokens(window)),
            None => format!("{} tks", human_tokens(total)),
        };
        segments.push(format!("{YELLOW}{token_str}{RESET}"));
    }

    // Session cost
    if let Some(ref cost) = input.cost {
        segments.push(format!("{PINK}${:.2}{RESET}", cost.total_cost_usd));
    }

    print!("{}", segments.join(&sep));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_tokens_small() {
        assert_eq!(human_tokens(847), "847");
    }

    #[test]
    fn test_human_tokens_low_k() {
        assert_eq!(human_tokens(1234), "1.2k");
    }

    #[test]
    fn test_human_tokens_mid_k() {
        assert_eq!(human_tokens(42000), "42k");
    }

    #[test]
    fn test_human_tokens_millions() {
        assert_eq!(human_tokens(1_523_400), "1.5M");
    }

    #[test]
    fn test_human_tokens_zero() {
        assert_eq!(human_tokens(0), "0");
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
