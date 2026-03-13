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
    used_percentage: Option<f64>,
}

struct Theme {
    green: &'static str,
    cyan: &'static str,
    purple: &'static str,
    yellow: &'static str,
    text: &'static str,
    separator: &'static str,
}

const RESET: &str = "\x1b[0m";

// Monokai Pro palette at ~60% brightness — for dark terminals
const DARK: Theme = Theme {
    green: "\x1b[38;2;122;158;86m",
    cyan: "\x1b[38;2;90;158;160m",
    purple: "\x1b[38;2;122;109;176m",
    yellow: "\x1b[38;2;176;154;66m",
    text: "\x1b[37m",
    separator: "\x1b[90m",
};

// Higher-saturation, darker values — for light terminals
const LIGHT: Theme = Theme {
    green: "\x1b[38;2;52;120;30m",
    cyan: "\x1b[38;2;24;110;120m",
    purple: "\x1b[38;2;88;70;154m",
    yellow: "\x1b[38;2;148;120;20m",
    text: "\x1b[90m",
    separator: "\x1b[37m",
};

fn resolve_theme(name: &str) -> &'static Theme {
    match name {
        "dark" => &DARK,
        "light" => &LIGHT,
        other => {
            eprintln!("ccline: unknown theme '{other}' (expected 'dark' or 'light')");
            std::process::exit(1);
        }
    }
}

fn parse_theme() -> &'static Theme {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--theme" {
            if let Some(val) = args.get(i + 1) {
                return resolve_theme(val);
            } else {
                eprintln!("ccline: --theme requires a value ('dark' or 'light')");
                std::process::exit(1);
            }
        }
        if let Some(val) = args[i].strip_prefix("--theme=") {
            return resolve_theme(val);
        }
        i += 1;
    }
    &DARK
}

fn git_info(path: &str, theme: &Theme) -> Option<String> {
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
        .is_some_and(|s| !s.is_empty());

    let dirty_marker = if dirty { "*" } else { "" };
    let purple = theme.purple;
    Some(format!("{purple}{}{dirty_marker}{RESET}", branch))
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
    let theme = parse_theme();

    let mut buf = String::new();
    if io::stdin().read_to_string(&mut buf).is_err() {
        return;
    }
    let input: Input = match serde_json::from_str(&buf) {
        Ok(v) => v,
        Err(_) => return,
    };

    let sep = format!(" {}|{RESET} ", theme.separator);
    let mut segments: Vec<String> = Vec::new();

    // Model name
    if let Some(ref model) = input.model {
        segments.push(format!("{}{}{RESET}", theme.green, model.display_name));
    }

    // Short path
    segments.push(format!(
        "{}{}{RESET}",
        theme.cyan,
        short_path(&input.workspace.current_dir)
    ));

    // Git branch + dirty
    if let Some(git) = git_info(&input.workspace.current_dir, theme) {
        segments.push(git);
    }

    // Context window usage
    if let Some(ref ctx) = input.context_window {
        if let (Some(pct), Some(window)) = (ctx.used_percentage, ctx.context_window_size) {
            let ctx_str = format!("{:.0}%/{} ctx", pct, human_tokens(window));
            segments.push(format!("{}{ctx_str}{RESET}", theme.yellow));
        }
    }

    // Token count + cost (combined)
    let total_tokens = input
        .context_window
        .as_ref()
        .map(|ctx| ctx.total_input_tokens + ctx.total_output_tokens);
    match (total_tokens, input.cost.as_ref()) {
        (Some(tks), Some(cost)) => {
            segments.push(format!(
                "{}{}/${:.2} tks{RESET}",
                theme.text,
                human_tokens(tks),
                cost.total_cost_usd
            ));
        }
        (Some(tks), None) => {
            segments.push(format!("{}{} tks{RESET}", theme.text, human_tokens(tks)));
        }
        (None, Some(cost)) => {
            segments.push(format!("{}${:.2}{RESET}", theme.text, cost.total_cost_usd));
        }
        _ => {}
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
}
