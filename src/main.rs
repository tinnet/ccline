use git2::Repository;
use serde::Deserialize;
use std::io::{self, Read};

#[derive(Deserialize)]
struct Input {
    workspace: Workspace,
    model: Option<Model>,
    effort: Option<Effort>,
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
struct Effort {
    level: String,
}

#[derive(Deserialize)]
struct Cost {
    total_cost_usd: f64,
}

#[derive(Deserialize)]
struct ContextWindow {
    context_window_size: Option<u64>,
    used_percentage: Option<f64>,
}

// Monokai Pro palette at ~60% brightness
const GREEN: &str = "\x1b[38;2;122;158;86m";
const CYAN: &str = "\x1b[38;2;90;158;160m";
const PURPLE: &str = "\x1b[38;2;122;109;176m";
const YELLOW: &str = "\x1b[38;2;176;154;66m";
const LIGHT_GRAY: &str = "\x1b[37m";
const GRAY: &str = "\x1b[90m";
const RED: &str = "\x1b[38;2;176;67;94m";
const ORANGE: &str = "\x1b[38;2;174;105;71m";
const RESET: &str = "\x1b[0m";

// Repo mark: shape x color = 72 combinations. Single-codepoint, one column wide,
// and without an emoji presentation, so terminals agree on their width.
const MARK_SHAPES: [&str; 12] = ["●", "■", "▲", "▼", "◆", "★", "✚", "✦", "✿", "◐", "✱", "⬢"];
const MARK_COLORS: [&str; 6] = [RED, ORANGE, YELLOW, GREEN, CYAN, PURPLE];

struct GitInfo {
    repo_name: Option<String>,
    branch: String,
}

fn git_info(path: &str) -> Option<GitInfo> {
    let repo = Repository::discover(path).ok()?;
    let repo_name = repo
        .workdir()
        .and_then(|dir| dir.file_name())
        .map(|name| name.to_string_lossy().into_owned());
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
    Some(GitInfo {
        repo_name,
        branch: format!("{PURPLE}{}{dirty_marker}{RESET}", branch),
    })
}

/// Deterministic shape + color for a repo name (32-bit FNV-1a over its UTF-8 bytes).
fn repo_mark(name: &str) -> String {
    let hash = name.bytes().fold(0x811c9dc5u32, |h, b| {
        (h ^ b as u32).wrapping_mul(0x01000193)
    }) as usize;
    let shape = MARK_SHAPES[hash % MARK_SHAPES.len()];
    let color = MARK_COLORS[hash / MARK_SHAPES.len() % MARK_COLORS.len()];
    format!("{color}{shape}{RESET}")
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

    // Model name + effort level
    if let Some(ref model) = input.model {
        let effort_suffix = input
            .effort
            .as_ref()
            .map(|e| format!(" {GRAY}({RESET}{YELLOW}{}{RESET}{GRAY}){RESET}", e.level))
            .unwrap_or_default();
        segments.push(format!(
            "{GREEN}{}{RESET}{effort_suffix}",
            model.display_name
        ));
    }

    let git = git_info(&input.workspace.current_dir);

    // Repo mark + short path
    let mark = git
        .as_ref()
        .and_then(|g| g.repo_name.as_deref())
        .map(|name| format!("{} ", repo_mark(name)))
        .unwrap_or_default();
    segments.push(format!(
        "{mark}{CYAN}{}{RESET}",
        short_path(&input.workspace.current_dir)
    ));

    // Git branch + dirty
    if let Some(git) = git {
        segments.push(git.branch);
    }

    // Context window usage
    if let Some(ref ctx) = input.context_window {
        if let (Some(pct), Some(window)) = (ctx.used_percentage, ctx.context_window_size) {
            let ctx_str = format!("{:.0}%/{} ctx", pct, human_tokens(window));
            segments.push(format!("{YELLOW}{ctx_str}{RESET}"));
        }
    }

    // Session cost
    if let Some(ref cost) = input.cost {
        segments.push(format!("{LIGHT_GRAY}${:.2}{RESET}", cost.total_cost_usd));
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
        assert_eq!(
            short_path("/Users/selkie/src/github.com/tinnet/ccline"),
            "tinnet/ccline"
        );
    }

    #[test]
    fn test_short_path_one_component() {
        assert_eq!(short_path("/tmp"), "tmp");
    }

    #[test]
    fn test_short_path_root() {
        assert_eq!(short_path("/"), "/");
    }

    // Pinned values: bench/ccline.sh implements the same hash and must agree.
    #[test]
    fn test_repo_mark_pinned() {
        assert_eq!(repo_mark("ccline"), format!("{YELLOW}◐{RESET}"));
        assert_eq!(repo_mark("dotfiles"), format!("{ORANGE}▼{RESET}"));
        assert_eq!(repo_mark("naïve"), format!("{ORANGE}✦{RESET}"));
    }

    #[test]
    fn test_repo_mark_anagrams_differ() {
        assert_ne!(repo_mark("api"), repo_mark("pia"));
    }
}
