use diesel::prelude::*;
use rask_api::endpoints::{NewTaskResponse, TaskListResponse};
use rask_api::models::{NewTask, Task, MODE_COMPLETED, MODE_PENDING};
use rask_api::schema::task;
use rocket::http::{ContentType, Status};
use rocket::local::blocking::Client;
use std::{env, panic};

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

/// Deletes all rows in the `task` table.
fn delete_all_tasks(conn: &PgConnection) {
    diesel::delete(task::table).execute(conn).unwrap();
}

/// Creates a new Task, verifies that it was creates successfully, and returns it.
fn create_task(client: &Client, task_to_create: &NewTask) -> Task {
    let response = client
        .post("/task")
        .header(ContentType::Form)
        .body(serde_urlencoded::to_string(task_to_create).unwrap())
        .dispatch();

    // The new task should have been created successfully.
    assert_eq!(response.status(), Status::Created);
    let new_task = response.into_json::<NewTaskResponse>().unwrap().task;
    assert_eq!(new_task.name, task_to_create.name);
    assert_eq!(new_task.project, task_to_create.project);
    assert_eq!(new_task.mode, MODE_PENDING.0);

    new_task
}

/// Marks `task_to_complete` as being completed.
fn mark_task_completed(client: &Client, task_to_complete: &Task) -> Task {
    let response = client
        .post(format!("/task/{}/complete", task_to_complete.id))
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

fn assert_tasks_endpoint_contains(
    client: &Client,
    uri: &str,
    task_list_response: &TaskListResponse,
) {
    let response = client.get(uri).dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.into_json::<TaskListResponse>().as_ref(),
        Some(task_list_response)
    );
}

#[test]
/// If we haven't created any tasks, then the tasks-getting endpoints should return an empty list.
fn test_get_tasks_when_no_tasks() {
    let client = get_client();
    let expected_response = TaskListResponse { tasks: vec![] };
    assert_tasks_endpoint_contains(&client, "/tasks/all", &expected_response);
    assert_tasks_endpoint_contains(&client, "/tasks/alive", &expected_response);
}

#[test]
/// The API should return a 404 response on a request to /task/<unused_id>.
fn test_get_task_by_id_404s_on_unknown_task() {
    let client = get_client();
    let response = client.get("/task/12345").dispatch();
    assert_eq!(response.status(), Status::NotFound);
}

#[test]
/// It should be possible to create a new task,
/// and the task should be visible via the other endpoints once it exists.
fn test_creating_task() {
    run_test(|| {
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
        let expected_response = TaskListResponse {
            tasks: vec![new_task.clone()],
        };
        assert_tasks_endpoint_contains(&client, "/tasks/all", &expected_response);
        assert_tasks_endpoint_contains(&client, "/tasks/alive", &expected_response);

        // The new task should appear in the get-task-by-id endpoint.
        let response = client.get(format!("/task/{}", new_task.id)).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_json::<Task>(), Some(new_task));
    });
}

#[test]
/// If a user tries to create a task with incorrectly-formatted JSON, we should return a 422 response.
fn test_creating_task_with_garbage_input() {
    run_test(|| {
        let client = get_client();
        let response = client
            .post("/task")
            .header(ContentType::Form)
            .body("foo=bar")
            .dispatch();

        assert_eq!(response.status(), Status::UnprocessableEntity);
    });
}

#[test]
/// Verify the behavior of completing a task.
fn test_completing_task() {
    run_test(|| {
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
        assert_tasks_endpoint_contains(
            &client,
            "/tasks/all",
            &TaskListResponse {
                tasks: vec![completed_task.clone()],
            },
        );
        // ...but it shouldn't appear in the get-alive-tasks endpoint.
        assert_tasks_endpoint_contains(
            &client,
            "/tasks/alive",
            &TaskListResponse { tasks: vec![] },
        );

        // The get-task-by-id endpoint's response should look just like `completed_task`.
        let response = client
            .get(format!("/task/{}", completed_task.id))
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_json::<Task>(), Some(completed_task));
    });
}

#[test]
/// Verify that we don't crash if a user tries to complete a task twice.
fn test_completing_twice() {
    run_test(|| {
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
    });
}

#[test]
/// Verify that we 404 if a user tries to complete a task that doesn't exist.
fn test_completing_nonexistent_task() {
    run_test(|| {
        let client = get_client();

        let response = client.post(format!("/task/{}/complete", 12345)).dispatch();
        assert_eq!(response.status(), Status::NotFound);
    });
}

#[test]
/// Test the behavior of tasks' .project field.
fn test_task_project_field() {
    run_test(|| {
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
    });
}

#[test]
/// Test the behavior of tasks' .priority field.
fn test_task_priority_field() {
    run_test(|| {
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
    });
}
