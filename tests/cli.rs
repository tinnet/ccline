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
