#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use crate::db::DBConn;
use dotenv::dotenv;
use rocket::fairing::AdHoc;
use rocket::figment::util::map;
use rocket::figment::value::{Map, Value};
use rocket::{launch, routes, Build, Rocket};
use std::env;

mod db;
mod db_queries;
mod endpoints;
mod models;
mod schema;

async fn run_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
    embed_migrations!();

    let conn = DBConn::get_one(&rocket).await.expect("database connection");
    conn.run(|c| embedded_migrations::run(c))
        .await
        .expect("diesel migrations");

    rocket
}

fn load_environment_variables() {
    if env::var("RUST_TESTING").is_ok() {
        dotenv::from_filename(".env.test").ok();
    } else {
        dotenv().ok();
    }
}

#[launch]
fn rocket() -> _ {
    load_environment_variables();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be defined");
    let db: Map<_, Value> = map! {
        "url" => db_url.into(),
        "pool_size" => 10.into()
    };

    rocket::custom(rocket::Config::figment().merge(("databases", map! {"rask_db" => db})))
        .mount(
            "/",
            routes![
                endpoints::get_tasks,
                endpoints::get_alive_tasks,
                endpoints::get_task_by_id,
                endpoints::create_task,
                endpoints::complete_task
            ],
        )
        .attach(DBConn::fairing())
        .attach(AdHoc::on_ignite("Diesel Migrations", run_migrations))
}

#[cfg(test)]
mod test {
    use super::{load_environment_variables, rocket};
    use crate::endpoints::{NewTaskResponse, TaskListResponse};
    use crate::models::{Task, MODE_PENDING};
    use crate::schema::task;
    use diesel::prelude::*;
    use rocket::http::{ContentType, Status};
    use rocket::local::blocking::Client;
    use std::{env, panic};

    /// Returns a connection to the database.
    fn get_db_conn() -> PgConnection {
        load_environment_variables();
        let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be defined");
        PgConnection::establish(&db_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", db_url))
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

    fn assert_all_tasks_endpoint_contains(client: &Client, task_list_response: &TaskListResponse) {
        let response = client.get("/tasks/all").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_json::<TaskListResponse>().as_ref(),
            Some(task_list_response)
        );
    }

    fn assert_alive_tasks_endpoint_contains(
        client: &Client,
        task_list_response: &TaskListResponse,
    ) {
        let response = client.get("/tasks/alive").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_json::<TaskListResponse>().as_ref(),
            Some(task_list_response)
        );
    }

    #[test]
    /// If we haven't created any tasks, then the tasks-getting endpoints should return an empty list.
    fn test_get_tasks_when_no_tasks() {
        let client = Client::tracked(rocket()).unwrap();
        let expected_response = TaskListResponse { tasks: vec![] };
        assert_all_tasks_endpoint_contains(&client, &expected_response);
        assert_alive_tasks_endpoint_contains(&client, &expected_response);
    }

    #[test]
    /// The API should return a 404 response on a request to /task/<unused_id>.
    fn test_get_task_by_id_404s_on_unknown_task() {
        let client = Client::tracked(rocket()).unwrap();
        let response = client.get("/task/12345").dispatch();
        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    /// It should be possible to create a new task,
    /// and the task should be visible via the other endpoints once it exists.
    fn test_creating_task() {
        run_test(|| {
            let client = Client::tracked(rocket()).unwrap();
            let response = client
                .post("/task")
                .header(ContentType::JSON)
                .body(r#"{"name": "this is a test task"}"#)
                .dispatch();

            // The new task should have been created successfully.
            assert_eq!(response.status(), Status::Created);
            let new_task = response.into_json::<NewTaskResponse>().unwrap().task;
            assert_eq!(new_task.name, "this is a test task");
            assert_eq!(new_task.mode, MODE_PENDING.0);

            // The new task should appear in the get-all-tasks endpoints.
            let expected_response = TaskListResponse {
                tasks: vec![new_task.clone()],
            };
            assert_all_tasks_endpoint_contains(&client, &expected_response);
            assert_alive_tasks_endpoint_contains(&client, &expected_response);

            // The new task should appear in the get-task-by-id endpoint.
            let response = client.get(format!("/task/{}", new_task.id)).dispatch();
            assert_eq!(response.status(), Status::Ok);
            assert_eq!(response.into_json::<Task>(), Some(new_task));
        });
    }
}
