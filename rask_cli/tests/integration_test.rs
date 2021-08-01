use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_no_args() {
    let mut cmd = Command::cargo_bin("rask_cli").unwrap();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("USAGE"));
}

#[test]
fn test_list() {
    let mut cmd = Command::cargo_bin("rask_cli").unwrap();
    cmd.arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Retrieved 0 tasks"));
}
