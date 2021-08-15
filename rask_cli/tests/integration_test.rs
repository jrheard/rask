use assert_cmd::Command;
use chrono::NaiveDate;
use diesel::prelude::*;
use predicates::prelude::*;
use rask_lib::models::NewTask;
use rask_lib::testing::{insert_example_api_token, run_test};
use regex::Regex;
use std::{env, panic, str};

const DB_URL: &str = "postgres://postgres:password@localhost:5001/rask";
const EXAMPLE_TOKEN: &str = "ef7025f8-1baa-4a20-96b5-8eff947f417d";

/// Returns a connection to the database.
fn get_db_conn() -> PgConnection {
    PgConnection::establish(DB_URL).unwrap_or_else(|_| panic!("Error connecting to {}", DB_URL))
}

fn get_cmd() -> Command {
    Command::cargo_bin("rask_cli").unwrap()
}

fn set_up_authorization() {
    let conn = get_db_conn();
    insert_example_api_token(&conn, EXAMPLE_TOKEN);

    env::set_var("RASK_API_TOKEN", EXAMPLE_TOKEN);
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

    let re = Regex::new(r"Task ([0-9]+):\n").unwrap();
    re.captures(str::from_utf8(&output.stdout).unwrap())
        .unwrap()[1]
        .to_string()
}

fn complete_task(id: &str) {
    let mut cmd = get_cmd();
    cmd.arg("complete")
        .arg(id)
        .assert()
        .success()
        .stdout(predicate::str::contains("Completed task"));
}

fn uncomplete_task(id: &str) {
    let mut cmd = get_cmd();
    cmd.arg("uncomplete")
        .arg(id)
        .assert()
        .success()
        .stdout(predicate::str::contains("Uncompleted task"));
}

#[test]
fn test_no_args() {
    run_test(
        || {
            set_up_authorization();

            let mut cmd = get_cmd();
            cmd.assert()
                .failure()
                .stderr(predicate::str::contains("USAGE"));
        },
        get_db_conn(),
    );
}

#[test]
fn test_list() {
    run_test(
        || {
            set_up_authorization();

            assert_list_output_contains("Retrieved 0 tasks");
        },
        get_db_conn(),
    );
}

#[test]
fn test_create_simple() {
    run_test(
        || {
            set_up_authorization();

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
        },
        get_db_conn(),
    )
}

#[test]
fn test_create_all_fields() {
    run_test(
        || {
            set_up_authorization();

            let id = create_task(NewTask {
                name: "clean litterbox".to_string(),
                project: Some("frank".to_string()),
                priority: Some("H".to_string()),
                due: Some(NaiveDate::from_ymd(2021, 7, 31)),
            });

            assert_list_output_contains("Retrieved 1 tasks");
            assert_list_output_contains("clean litterbox");

            assert_info_output_contains(&id, &format!("Task {}", id));
            assert_info_output_contains(&id, "clean litterbox");
            assert_info_output_contains(&id, "Project:\tfrank");
            assert_info_output_contains(&id, "Priority:\tH");
            assert_info_output_contains(&id, "Due:\t\t07/31/2021");
        },
        get_db_conn(),
    )
}

#[test]
fn test_completing_task() {
    run_test(
        || {
            set_up_authorization();

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
                .stdout(predicate::str::contains("Completed task"));

            assert_list_output_contains("Retrieved 0 tasks");
        },
        get_db_conn(),
    );
}

#[test]
fn test_modify_task() {
    run_test(
        || {
            set_up_authorization();

            let id = create_task(NewTask {
                name: "clean litterbox".to_string(),
                project: Some("frank".to_string()),
                priority: Some("H".to_string()),
                due: Some(NaiveDate::from_ymd(2021, 7, 31)),
            });

            let mut cmd = get_cmd();
            // Change the name and project, leave the priority as-is, and delete the due date.
            cmd.args(&[
                "modify",
                &id,
                "dust shelves",
                "--project",
                "house",
                "--due",
                "none",
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("Updated task"));

            assert_info_output_contains(&id, &format!("Task {}", id));
            assert_info_output_contains(&id, "dust shelves");
            assert_info_output_contains(&id, "Project:\thouse");
            assert_info_output_contains(&id, "Priority:\tH");
            assert_info_output_contains(&id, "Due:\t\tN/A");
        },
        get_db_conn(),
    );
}

#[test]
fn test_api_token_handling() {
    run_test(
        || {
            let conn = get_db_conn();
            insert_example_api_token(&conn, EXAMPLE_TOKEN);

            // Run the CLI with no RASK_API_TOKEN env var set.
            let mut cmd = get_cmd();
            cmd.arg("list").assert().failure();

            for bad_token in [
                "",
                "foo",
                "Bearer foo",
                "Bearer foo bar baz",
                "Bearer 309dcde0-5bc4-4e9f-a32a-b5bbee54eb81",
            ] {
                env::set_var("RASK_API_TOKEN", bad_token);

                let mut cmd = get_cmd();
                cmd.arg("list").assert().failure();
            }
        },
        get_db_conn(),
    );
}

#[test]
fn test_uncompleting_task() {
    run_test(
        || {
            set_up_authorization();

            let id = create_task(NewTask {
                name: "hello there".to_string(),
                project: None,
                priority: None,
                due: None,
            });

            assert_list_output_contains("Retrieved 1 tasks");
            assert_list_output_contains("hello there");
            assert_info_output_contains(&id, "pending");

            // Uncompleting a pending task leaves it as-is.
            uncomplete_task(&id);
            assert_list_output_contains("hello there");
            assert_info_output_contains(&id, "pending");

            complete_task(&id);
            assert_info_output_contains(&id, "completed");
            assert_list_output_contains("Retrieved 0 tasks");

            uncomplete_task(&id);
            assert_list_output_contains("hello there");
            assert_info_output_contains(&id, "pending");
        },
        get_db_conn(),
    );
}
