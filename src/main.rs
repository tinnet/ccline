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
    let user = std::env::var("USER").unwrap_or_else(|_| "?".into());
    let host = gethostname::gethostname();
    let host = host.to_string_lossy();
    let host = host.strip_suffix(".local").unwrap_or(&host);

    print!(
        "{}@{} \x1b[34m{}\x1b[0m",
        user, host, input.workspace.current_dir
    );
}
