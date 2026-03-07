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
