use assert_cmd::Command;
use chrono::NaiveDate;
use diesel::prelude::*;
use predicates::prelude::*;
use rask_lib::{models::NewTask, schema::task};
use regex::Regex;
use std::{panic, str};

const DB_URL: &str = "postgres://postgres:password@localhost:5001/rask";

/// Returns a connection to the database.
fn get_db_conn() -> PgConnection {
    PgConnection::establish(DB_URL).unwrap_or_else(|_| panic!("Error connecting to {}", DB_URL))
}

/// Deletes all rows in the `task` table.
fn delete_all_tasks(conn: &PgConnection) {
    diesel::delete(task::table).execute(conn).unwrap();
}

/// Runs a chunk of test code in a setup/teardown block.
/// Via https://medium.com/@ericdreichert/test-setup-and-teardown-in-rust-without-a-framework-ba32d97aa5ab .
fn run_test<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    let conn = get_db_conn();

    let result = panic::catch_unwind(|| test());

    delete_all_tasks(&conn);

    assert!(result.is_ok());
}

fn get_cmd() -> Command {
    Command::cargo_bin("rask_cli").unwrap()
}

fn assert_list_output_contains(expected_output: &str) {
    let mut cmd = get_cmd();
    cmd.arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains(expected_output));
}

fn assert_info_output_contains(task_id: &str, expected_output: &str) {
    let mut cmd = get_cmd();
    cmd.arg("info")
        .arg(task_id)
        .assert()
        .success()
        .stdout(predicate::str::contains(expected_output));
}

fn create_task(input: NewTask) -> String {
    let mut cmd = get_cmd();
    let mut cmd = cmd.arg("create").arg(input.name);

    if let Some(project) = input.project {
        cmd = cmd.arg("--project").arg(project);
    }
    if let Some(priority) = input.priority {
        cmd = cmd.arg("--priority").arg(priority);
    }
    if let Some(due) = input.due {
        cmd = cmd
            .arg("--due")
            .arg(due.format(rask_cli::DATE_FORMAT).to_string());
    }

    let assert = cmd.assert().success();
    let output = assert.get_output();

    let re = Regex::new(r"Successfully created task with ID ([0-9]+).\n").unwrap();
    re.captures(str::from_utf8(&output.stdout).unwrap())
        .unwrap()[1]
        .to_string()
}

#[test]
fn test_no_args() {
    let mut cmd = get_cmd();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("USAGE"));
}

#[test]
fn test_list() {
    assert_list_output_contains("Retrieved 0 tasks");
}

#[test]
fn test_create_simple() {
    run_test(|| {
        let id = create_task(NewTask {
            name: "hello there".to_string(),
            project: None,
            priority: None,
            due: None,
        });

        assert_list_output_contains("Retrieved 1 tasks");
        assert_list_output_contains("hello there");

        assert_info_output_contains(&id, &format!("Task {}", id));
        assert_info_output_contains(&id, "hello there");
    })
}

#[test]
fn test_create_all_fields() {
    run_test(|| {
        let id = create_task(NewTask {
            name: "clean litterbox".to_string(),
            project: Some("frank".to_string()),
            priority: Some("H".to_string()),
            due: Some(NaiveDate::from_ymd(2021, 7, 31).and_hms(0, 0, 0)),
        });

        assert_list_output_contains("Retrieved 1 tasks");
        assert_list_output_contains("clean litterbox");

        assert_info_output_contains(&id, &format!("Task {}", id));
        assert_info_output_contains(&id, "clean litterbox");
        assert_info_output_contains(&id, "Project:\tfrank");
        assert_info_output_contains(&id, "Priority:\tH");
        assert_info_output_contains(&id, "Due:\t\t07/31/2021");
    })
}

#[test]
fn test_completing_task() {
    let id = create_task(NewTask {
        name: "hello there".to_string(),
        project: None,
        priority: None,
        due: None,
    });

    assert_list_output_contains("Retrieved 1 tasks");
    assert_list_output_contains("hello there");

    let mut cmd = get_cmd();
    cmd.arg("complete")
        .arg(id)
        .assert()
        .success()
        .stdout(predicate::str::contains("Success"));

    assert_list_output_contains("Retrieved 0 tasks");
}
