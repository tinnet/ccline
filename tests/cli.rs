use assert_cmd::cargo_bin_cmd;
use predicates::prelude::*;

fn full_json() -> String {
    let cwd = std::env::current_dir().unwrap();
    format!(
        r#"{{"workspace":{{"current_dir":"{}","project_dir":"{}","added_dirs":[]}},"model":{{"id":"claude-opus-4-6","display_name":"Opus"}},"cost":{{"total_cost_usd":0.12}},"context_window":{{"total_input_tokens":30000,"total_output_tokens":12000,"context_window_size":200000,"used_percentage":10.0}}}}"#,
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
    cmd.args(["--theme", "dark"]);
    cmd.write_stdin(full_json());
    // Dark theme purple: rgb(122,109,176) — used for git branch
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
        .stdout(predicate::str::contains("10%/200k ctx"));
}

#[test]
fn shows_cost() {
    let mut cmd = cargo_bin_cmd!("ccline");
    cmd.write_stdin(full_json());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("42k/$0.12 tks"));
}

#[test]
fn shows_pipe_separators() {
    let mut cmd = cargo_bin_cmd!("ccline");
    cmd.write_stdin(full_json());
    cmd.assert().success().stdout(predicate::str::contains("|"));
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

#[test]
fn theme_dark_is_default() {
    let mut cmd = cargo_bin_cmd!("ccline");
    cmd.write_stdin(full_json());
    // Dark theme uses Monokai Pro purple: rgb(122,109,176)
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\x1b[38;2;122;109;176m"));
}

#[test]
fn theme_light_flag() {
    let mut cmd = cargo_bin_cmd!("ccline");
    cmd.args(["--theme", "light"]);
    cmd.write_stdin(full_json());
    // Light theme: green rgb(52,120,30), purple rgb(88,70,154)
    // Dark theme purple rgb(122,109,176) must be absent
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\x1b[38;2;52;120;30m"))
        .stdout(predicate::str::contains("\x1b[38;2;88;70;154m"))
        .stdout(predicate::str::contains("\x1b[38;2;122;109;176m").not());
}

#[test]
fn theme_dark_explicit() {
    let mut cmd = cargo_bin_cmd!("ccline");
    cmd.args(["--theme", "dark"]);
    cmd.write_stdin(full_json());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\x1b[38;2;122;109;176m"));
}

#[test]
fn theme_bad_value() {
    let mut cmd = cargo_bin_cmd!("ccline");
    cmd.args(["--theme", "nope"]);
    cmd.write_stdin(full_json());
    cmd.assert().failure();
}

#[test]
fn theme_equals_syntax() {
    let mut cmd = cargo_bin_cmd!("ccline");
    cmd.args(["--theme=light"]);
    cmd.write_stdin(full_json());
    // Light theme green rgb(52,120,30)
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\x1b[38;2;52;120;30m"));
}

#[test]
fn theme_missing_value() {
    let mut cmd = cargo_bin_cmd!("ccline");
    cmd.args(["--theme"]);
    cmd.write_stdin(full_json());
    cmd.assert().failure();
}
