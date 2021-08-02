use chrono::NaiveDateTime;
use diesel::prelude::*;
use rask_lib::models::{NewTask, Task, MODE_COMPLETED, MODE_PENDING};
use rask_lib::testing::{insert_example_api_token, run_test};
use rocket::http::{ContentType, Header, Status};
use rocket::local::blocking::{Client, LocalRequest};
use std::{env, panic};

const EXAMPLE_TOKEN: &str = "9fc51cf8-461c-4092-a705-476c98e358cb";

/// Returns a local blocking Rocket Client.
fn get_client() -> Client {
    Client::tracked(rask_api::assemble_rocket()).unwrap()
}

/// Returns a connection to the database.
fn get_db_conn() -> PgConnection {
    rask_api::load_environment_variables();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be defined");
    PgConnection::establish(&db_url).unwrap_or_else(|_| panic!("Error connecting to {}", db_url))
}

trait Authorizable {
    fn add_authorization_header(self) -> Self;
}

impl<'a> Authorizable for LocalRequest<'a> {
    fn add_authorization_header(self) -> Self {
        let conn = get_db_conn();
        insert_example_api_token(&conn, EXAMPLE_TOKEN);

        self.header(Header::new(
            "Authorization",
            format!("Bearer {}", EXAMPLE_TOKEN),
        ))
    }
}

/// Creates a new Task, verifies that it was creates successfully, and returns it.
fn create_task(client: &Client, task_to_create: &NewTask) -> Task {
    let response = client
        .post("/task")
        .header(ContentType::Form)
        .add_authorization_header()
        .body(serde_urlencoded::to_string(task_to_create).unwrap())
        .dispatch();

    // The new task should have been created successfully.
    assert_eq!(response.status(), Status::Created);
    let new_task = response.into_json::<Task>().unwrap();
    assert_eq!(new_task.name, task_to_create.name);
    assert_eq!(new_task.project, task_to_create.project);
    assert_eq!(new_task.mode, MODE_PENDING.0);

    new_task
}

/// Marks `task_to_complete` as being completed.
fn mark_task_completed(client: &Client, task_to_complete: &Task) -> Task {
    let response = client
        .post(format!("/task/{}/complete", task_to_complete.id))
        .add_authorization_header()
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

    let completed_task = response.into_json::<Task>().unwrap();
    assert_eq!(
        completed_task,
        Task {
            name: task_to_complete.name.clone(),
            id: task_to_complete.id,
            mode: MODE_COMPLETED.0.to_string(),
            project: task_to_complete.project.clone(),
            priority: task_to_complete.priority.clone(),
            due: task_to_complete.due
        }
    );

    completed_task
}

fn get_example_datetime() -> NaiveDateTime {
    chrono::NaiveDate::from_ymd(2021, 7, 25).and_hms(23, 56, 4)
}

fn assert_tasks_endpoint_contains(client: &Client, uri: &str, task_list_response: &[Task]) {
    let response = client.get(uri).add_authorization_header().dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        &response.into_json::<Vec<Task>>().unwrap(),
        task_list_response
    );
}

#[test]
/// If we haven't created any tasks, then the tasks-getting endpoints should return an empty list.
fn test_get_tasks_when_no_tasks() {
    run_test(
        || {
            let client = get_client();
            let expected_response = vec![];
            assert_tasks_endpoint_contains(&client, "/tasks/all", &expected_response);
            assert_tasks_endpoint_contains(&client, "/tasks/alive", &expected_response);
        },
        get_db_conn(),
    );
}

#[test]
/// The API should return a 404 response on a request to /task/<unused_id>.
fn test_get_task_by_id_404s_on_unknown_task() {
    run_test(
        || {
            let client = get_client();
            let response = client
                .get("/task/12345")
                .add_authorization_header()
                .dispatch();
            assert_eq!(response.status(), Status::NotFound);
        },
        get_db_conn(),
    );
}

#[test]
/// It should be possible to create a new task,
/// and the task should be visible via the other endpoints once it exists.
fn test_creating_task() {
    run_test(
        || {
            let client = get_client();
            let new_task = create_task(
                &client,
                &NewTask {
                    name: "this is a test task".to_string(),
                    project: None,
                    priority: None,
                    due: None,
                },
            );

            // The new task should appear in the get-all-tasks endpoints.
            let expected_response = vec![new_task.clone()];
            assert_tasks_endpoint_contains(&client, "/tasks/all", &expected_response);
            assert_tasks_endpoint_contains(&client, "/tasks/alive", &expected_response);

            // The new task should appear in the get-task-by-id endpoint.
            let response = client
                .get(format!("/task/{}", new_task.id))
                .add_authorization_header()
                .dispatch();
            assert_eq!(response.status(), Status::Ok);
            assert_eq!(response.into_json::<Task>(), Some(new_task));
        },
        get_db_conn(),
    );
}

#[test]
/// If a user tries to create a task with incorrectly-formatted JSON, we should return a 422 response.
fn test_creating_task_with_garbage_input() {
    run_test(
        || {
            let client = get_client();
            let response = client
                .post("/task")
                .add_authorization_header()
                .header(ContentType::Form)
                .body("foo=bar")
                .dispatch();

            assert_eq!(response.status(), Status::UnprocessableEntity);
        },
        get_db_conn(),
    );
}

#[test]
/// Verify the behavior of completing a task.
fn test_completing_task() {
    run_test(
        || {
            let client = get_client();
            let new_task = create_task(
                &client,
                &NewTask {
                    name: "this is a test task".to_string(),
                    project: None,
                    priority: None,
                    due: None,
                },
            );

            let completed_task = mark_task_completed(&client, &new_task);

            // The completed task should appear in the get-all-tasks endpoint...
            assert_tasks_endpoint_contains(&client, "/tasks/all", &[completed_task.clone()]);
            // ...but it shouldn't appear in the get-alive-tasks endpoint.
            assert_tasks_endpoint_contains(&client, "/tasks/alive", &[]);

            // The get-task-by-id endpoint's response should look just like `completed_task`.
            let response = client
                .get(format!("/task/{}", completed_task.id))
                .add_authorization_header()
                .dispatch();
            assert_eq!(response.status(), Status::Ok);
            assert_eq!(response.into_json::<Task>(), Some(completed_task));
        },
        get_db_conn(),
    );
}

#[test]
/// Verify that we don't crash if a user tries to complete a task twice.
fn test_completing_twice() {
    run_test(
        || {
            let client = get_client();
            let new_task = create_task(
                &client,
                &NewTask {
                    name: "this is a test task".to_string(),
                    project: None,
                    priority: None,
                    due: None,
                },
            );

            let completed_task = mark_task_completed(&client, &new_task);
            mark_task_completed(&client, &completed_task);
        },
        get_db_conn(),
    );
}

#[test]
/// Verify that we 404 if a user tries to complete a task that doesn't exist.
fn test_completing_nonexistent_task() {
    run_test(
        || {
            let client = get_client();

            let response = client
                .post(format!("/task/{}/complete", 12345))
                .add_authorization_header()
                .dispatch();
            assert_eq!(response.status(), Status::NotFound);
        },
        get_db_conn(),
    );
}

#[test]
/// Test the behavior of tasks' .project field.
fn test_task_project_field() {
    run_test(
        || {
            let client = get_client();

            // Creating a task with a one-word project name should work fine.
            create_task(
                &client,
                &NewTask {
                    name: "clean dishes".to_string(),
                    project: Some("house".to_string()),
                    priority: None,
                    due: None,
                },
            );

            // Creating a task with a multi-word project should give a 422.
            let response = client
                .post("/task")
                .header(ContentType::Form)
                .add_authorization_header()
                .body(
                    serde_urlencoded::to_string(&NewTask {
                        name: "clean dishes".to_string(),
                        project: Some("multi word project".to_string()),
                        priority: None,
                        due: None,
                    })
                    .unwrap(),
                )
                .dispatch();

            assert_eq!(response.status(), Status::UnprocessableEntity);
        },
        get_db_conn(),
    );
}

#[test]
/// Test the behavior of tasks' .priority field.
fn test_task_priority_field() {
    run_test(
        || {
            let client = get_client();

            // Creating a task with a valid priority value should work fine.
            create_task(
                &client,
                &NewTask {
                    name: "clean dishes".to_string(),
                    project: Some("frank".to_string()),
                    priority: Some("M".to_string()),
                    due: None,
                },
            );

            // Creating a task with a junk priority should give a 422.
            let response = client
                .post("/task")
                .header(ContentType::Form)
                .add_authorization_header()
                .body(
                    serde_urlencoded::to_string(&NewTask {
                        name: "clean dishes".to_string(),
                        project: Some("frank".to_string()),
                        priority: Some("garbage".to_string()),
                        due: None,
                    })
                    .unwrap(),
                )
                .dispatch();

            assert_eq!(response.status(), Status::UnprocessableEntity);
        },
        get_db_conn(),
    );
}

#[test]
/// Test the behavior of tasks' .due field.
fn test_task_due_field() {
    run_test(
        || {
            let client = get_client();

            // Creating a task with a valid due timestamp should work fine.
            let response = client
                .post("/task")
                .header(ContentType::Form)
                .add_authorization_header()
                .body(
                    // 2021-07-25T23:56:04
                    "name=clean+dishes&project=house&due=2021-07-25T23%3A56%3A04",
                )
                .dispatch();

            // The new task should have been created successfully.
            assert_eq!(response.status(), Status::Created);
            let new_task = response.into_json::<Task>().unwrap();

            assert_eq!(
                new_task,
                Task {
                    id: new_task.id,
                    name: "clean dishes".to_string(),
                    project: Some("house".to_string()),
                    mode: MODE_PENDING.0.to_string(),
                    priority: None,
                    due: Some(get_example_datetime())
                }
            );

            // Creating a task with a junk due date should give the task a null due date.
            let response = client
                .post("/task")
                .header(ContentType::Form)
                .add_authorization_header()
                .body(
                    // 2021-07-25 23:56:04
                    "name=clean+dishes&project=house&due=garbage",
                )
                .dispatch();

            // The new task should have been created successfully, but have no due date.
            assert_eq!(response.status(), Status::Created);
            let new_task = response.into_json::<Task>().unwrap();
            assert_eq!(
                new_task,
                Task {
                    id: new_task.id,
                    name: "clean dishes".to_string(),
                    project: Some("house".to_string()),
                    mode: MODE_PENDING.0.to_string(),
                    priority: None,
                    due: None
                }
            );
        },
        get_db_conn(),
    );
}

#[test]
/// The /task/<task_id>/edit endpoint should let users edit a task.
fn test_editing_task() {
    run_test(
        || {
            let client = get_client();
            let new_task = create_task(
                &client,
                &NewTask {
                    name: "this is a test task".to_string(),
                    project: None,
                    priority: None,
                    due: None,
                },
            );

            let response = client
                .post(format!("/task/{}/edit", new_task.id))
                .header(ContentType::Form)
                .add_authorization_header()
                .body(
                    serde_urlencoded::to_string(NewTask {
                        name: "clean litterbox".to_string(),
                        project: Some("frank".to_string()),
                        priority: Some("H".to_string()),
                        due: Some(get_example_datetime()),
                    })
                    .unwrap(),
                )
                .dispatch();

            assert_eq!(response.status(), Status::Ok);

            let updated_task = response.into_json::<Task>().unwrap();
            assert_eq!(
                updated_task,
                Task {
                    id: new_task.id,
                    name: "clean litterbox".to_string(),
                    mode: MODE_PENDING.0.to_string(),
                    project: Some("frank".to_string()),
                    priority: Some("H".to_string()),
                    due: Some(get_example_datetime()),
                }
            );
        },
        get_db_conn(),
    );
}

#[test]
/// The healthcheck endpoint should return a 200.
fn test_healthcheck_endpoint() {
    let client = get_client();
    let response = client.get("/healthcheck").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string(), Some("hello!".to_string()));
}

#[test]
/// Requests should be 400'd or 401'd if they don't specify a valid API token.
fn test_api_token_handling() {
    run_test(
        || {
            let client = get_client();

            let conn = get_db_conn();
            insert_example_api_token(&conn, EXAMPLE_TOKEN);

            // Make a request with no Authorization header.
            let response = client.get("/tasks/alive").dispatch();
            assert_eq!(response.status(), Status::Unauthorized);

            for bad_token in [
                "",
                "foo",
                "Bearer foo",
                "Bearer foo bar baz",
                "Bearer 309dcde0-5bc4-4e9f-a32a-b5bbee54eb81",
            ] {
                let response = client
                    .get("/tasks/alive")
                    .header(Header::new(
                        "Authorization",
                        format!("Bearer {}", bad_token),
                    ))
                    .dispatch();

                assert_ne!(response.status(), Status::Ok);
            }
        },
        get_db_conn(),
    );
}
