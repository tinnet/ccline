use assert_cmd::cargo_bin_cmd;
use predicates::prelude::*;

fn full_json() -> String {
    let cwd = std::env::current_dir().unwrap();
    format!(
        r#"{{"workspace":{{"current_dir":"{}","project_dir":"{}","added_dirs":[]}},"model":{{"id":"claude-opus-4-6","display_name":"Opus"}},"cost":{{"total_cost_usd":0.12}},"context_window":{{"total_input_tokens":30000,"total_output_tokens":12000}}}}"#,
        cwd.display(),
        cwd.display()
    )
}

fn minimal_json() -> &'static str {
    r#"{"workspace":{"current_dir":"/tmp/foo/bar"}}"#
}

#[test]
fn shows_model_name() {
    let mut cmd = cargo_bin_cmd!("ccline");
    cmd.write_stdin(full_json());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Opus"));
}

#[test]
fn shows_short_path() {
    let mut cmd = cargo_bin_cmd!("ccline");
    cmd.write_stdin(minimal_json());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("foo/bar"));
}

#[test]
fn shows_git_branch_in_repo() {
    let mut cmd = cargo_bin_cmd!("ccline");
    cmd.write_stdin(full_json());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\x1b[38;2;122;109;176m"));
}

#[test]
fn no_git_outside_repo() {
    let mut cmd = cargo_bin_cmd!("ccline");
    cmd.write_stdin(minimal_json());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\x1b[38;2;122;109;176m").not());
}

#[test]
fn shows_token_count() {
    let mut cmd = cargo_bin_cmd!("ccline");
    cmd.write_stdin(full_json());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("42k tks"));
}

#[test]
fn shows_cost() {
    let mut cmd = cargo_bin_cmd!("ccline");
    cmd.write_stdin(full_json());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("$0.12"));
}

#[test]
fn shows_pipe_separators() {
    let mut cmd = cargo_bin_cmd!("ccline");
    cmd.write_stdin(full_json());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("|"));
}

#[test]
fn works_with_minimal_json() {
    let mut cmd = cargo_bin_cmd!("ccline");
    cmd.write_stdin(minimal_json());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("foo/bar"));
}

#[test]
fn no_user_host() {
    let user = std::env::var("USER").unwrap_or_default();
    let mut cmd = cargo_bin_cmd!("ccline");
    cmd.write_stdin(full_json());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(&format!("{}@", user)).not());
}
