use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn parses_workspace_cwd() {
    let input = r#"{"workspace":{"current_dir":"/tmp/test","project_dir":"/tmp/test","added_dirs":[]}}"#;
    let mut cmd = Command::cargo_bin("cld-sts-line").unwrap();
    cmd.write_stdin(input);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("/tmp/test"));
}

#[test]
fn includes_user_and_host() {
    let input = r#"{"workspace":{"current_dir":"/tmp/test","project_dir":"/tmp/test","added_dirs":[]}}"#;
    let user = std::env::var("USER").unwrap();
    let mut cmd = Command::cargo_bin("cld-sts-line").unwrap();
    cmd.write_stdin(input);
    cmd.assert()
        .success()
        .stdout(predicate::str::starts_with(&format!("{}@", user)));
}

#[test]
fn cwd_is_blue() {
    let input = r#"{"workspace":{"current_dir":"/tmp/test","project_dir":"/tmp/test","added_dirs":[]}}"#;
    let mut cmd = Command::cargo_bin("cld-sts-line").unwrap();
    cmd.write_stdin(input);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\x1b[34m/tmp/test\x1b[0m"));
}

#[test]
fn shows_git_branch() {
    // Use the project's own repo as test input — we know it's a git repo
    let cwd = std::env::current_dir().unwrap();
    let input = format!(
        r#"{{"workspace":{{"current_dir":"{}","project_dir":"{}","added_dirs":[]}}}}"#,
        cwd.display(),
        cwd.display()
    );
    let mut cmd = Command::cargo_bin("cld-sts-line").unwrap();
    cmd.write_stdin(input.as_str());
    // Should contain the gray ANSI code for a branch name
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\x1b[90m"));
}

#[test]
fn works_without_git() {
    let input = r#"{"workspace":{"current_dir":"/tmp","project_dir":"/tmp","added_dirs":[]}}"#;
    let mut cmd = Command::cargo_bin("cld-sts-line").unwrap();
    cmd.write_stdin(input);
    cmd.assert()
        .success()
        // Should NOT contain git ANSI codes
        .stdout(predicate::str::contains("\x1b[90m").not());
}
