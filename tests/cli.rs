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
    cmd.write_stdin(full_json());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\x1b[38;2;122;109;176m"));
}

const MARK_SHAPES: [&str; 12] = ["●", "■", "▲", "▼", "◆", "★", "✚", "✦", "✿", "◐", "✱", "⬢"];

fn has_repo_mark(out: &str) -> bool {
    MARK_SHAPES.iter().any(|shape| out.contains(shape))
}

#[test]
fn shows_repo_mark_in_repo() {
    let mut cmd = cargo_bin_cmd!("ccline");
    cmd.write_stdin(full_json());
    cmd.assert()
        .success()
        .stdout(predicate::function(has_repo_mark));
}

#[test]
fn shows_git_info_in_subdirectory() {
    let subdir = std::env::current_dir().unwrap().join("src");
    let mut cmd = cargo_bin_cmd!("ccline");
    cmd.write_stdin(format!(
        r#"{{"workspace":{{"current_dir":"{}"}}}}"#,
        subdir.display()
    ));
    cmd.assert()
        .success()
        .stdout(predicate::function(has_repo_mark))
        .stdout(predicate::str::contains("\x1b[38;2;122;109;176m"));
}

#[test]
fn no_git_outside_repo() {
    let mut cmd = cargo_bin_cmd!("ccline");
    cmd.write_stdin(minimal_json());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\x1b[38;2;122;109;176m").not())
        .stdout(predicate::function(|out: &str| !has_repo_mark(out)));
}

#[test]
fn shows_context_usage() {
    let mut cmd = cargo_bin_cmd!("ccline");
    cmd.write_stdin(full_json());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("10%/200k ctx"));
}

#[test]
fn shows_rate_limits() {
    let mut cmd = cargo_bin_cmd!("ccline");
    cmd.write_stdin(
        r#"{"workspace":{"current_dir":"/tmp/foo/bar"},"rate_limits":{"five_hour":{"used_percentage":23.5,"resets_at":1738425600},"seven_day":{"used_percentage":81.2,"resets_at":1738857600}}}"#,
    );
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "\x1b[90m5h\x1b[0m \x1b[38;2;122;158;86m24%\x1b[0m",
        ))
        .stdout(predicate::str::contains(
            "\x1b[90m7d\x1b[0m \x1b[38;2;176;67;94m81%\x1b[0m",
        ));
}

#[test]
fn shows_only_present_rate_limit_windows() {
    let mut cmd = cargo_bin_cmd!("ccline");
    cmd.write_stdin(
        r#"{"workspace":{"current_dir":"/tmp/foo/bar"},"rate_limits":{"seven_day":{"used_percentage":41.2,"resets_at":1738857600}}}"#,
    );
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("5h").not())
        .stdout(predicate::str::contains("41%"));
}

#[test]
fn no_rate_limits_without_subscription() {
    let mut cmd = cargo_bin_cmd!("ccline");
    cmd.write_stdin(full_json());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("5h").not())
        .stdout(predicate::str::contains("7d").not());
}

fn limits_json(five_pct: f64, five_reset_in: i64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    format!(
        r#"{{"workspace":{{"current_dir":"/tmp/foo/bar"}},"rate_limits":{{"five_hour":{{"used_percentage":{},"resets_at":{}}}}}}}"#,
        five_pct,
        now + five_reset_in
    )
}

#[test]
fn shows_reset_time_when_red() {
    let mut cmd = cargo_bin_cmd!("ccline");
    // 1h20m30s: the extra 30s keeps the minutes stable while the test runs
    cmd.write_stdin(limits_json(85.0, 3600 + 20 * 60 + 30));
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("85%\x1b[0m \x1b[90m↻1h20m\x1b[0m"));
}

#[test]
fn no_reset_time_below_red() {
    let mut cmd = cargo_bin_cmd!("ccline");
    cmd.write_stdin(limits_json(79.0, 3600));
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("↻").not());
}

#[test]
fn no_reset_time_when_already_past() {
    let mut cmd = cargo_bin_cmd!("ccline");
    cmd.write_stdin(limits_json(90.0, -60));
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("90%"))
        .stdout(predicate::str::contains("↻").not());
}

#[test]
fn shows_cost() {
    let mut cmd = cargo_bin_cmd!("ccline");
    cmd.write_stdin(full_json());
    cmd.assert().success().stdout(predicate::str::contains(
        "\x1b[90m~\x1b[0m\x1b[37m$0.12\x1b[0m",
    ));
}

#[test]
fn no_session_token_count() {
    let mut cmd = cargo_bin_cmd!("ccline");
    cmd.write_stdin(full_json());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("tks").not());
}

#[test]
fn hides_context_when_percentage_null() {
    let mut cmd = cargo_bin_cmd!("ccline");
    cmd.write_stdin(
        r#"{"workspace":{"current_dir":"/tmp/foo/bar"},"cost":{"total_cost_usd":0.0},"context_window":{"total_input_tokens":0,"total_output_tokens":0,"context_window_size":200000,"used_percentage":null}}"#,
    );
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("ctx").not())
        .stdout(predicate::str::contains("$0.00"));
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
