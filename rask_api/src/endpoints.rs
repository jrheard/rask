use crate::db::DBConn;
use crate::models::{NewTask, Task};
use crate::schema::task;
use diesel::prelude::*;
use rocket::http::{ContentType, Status};
use rocket::response::status::Created;
use rocket::response::{Responder, Response};
use rocket::serde::json::Json;
use rocket::{get, post};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RaskApiError {
    #[error(transparent)]
    DatabaseError(#[from] diesel::result::Error),
}

impl<'r> Responder<'r, 'static> for RaskApiError {
    /// Returns a 500 "internal server error" response.
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        let body = format!("Error: {}", self);
        let response = Response::build()
            .status(Status::InternalServerError)
            .header(ContentType::Plain)
            .sized_body(body.len(), Cursor::new(body))
            .finalize();

        Ok(response)
    }
}

type Result<T, E = RaskApiError> = std::result::Result<T, E>;

#[get("/task/<task_id>")]
pub async fn get_task_by_id(db: DBConn, task_id: i32) -> Result<Json<Task>> {
    db.run(move |conn| task::table.filter(task::id.eq(task_id)).first(conn))
        .await
        .map(Json)
        .map_err(|e| RaskApiError::DatabaseError(e))
}

#[derive(Serialize)]
pub struct TaskListResponse {
    tasks: Vec<Task>,
}

#[get("/tasks")]
pub async fn get_tasks(db: DBConn) -> Result<Json<TaskListResponse>> {
    let tasks = db.run(move |conn| task::table.load(conn)).await?;

    Ok(Json(TaskListResponse { tasks }))
}

#[derive(Deserialize)]
pub struct TaskJSON {
    name: String,
}

#[derive(Serialize)]
pub struct NewTaskResponse {
    task: Task,
}

#[post("/task", format = "json", data = "<task_json>")]
pub async fn create_task(
    db: DBConn,
    task_json: Json<TaskJSON>,
) -> Result<Created<Json<NewTaskResponse>>> {
    let new_task = db
        .run(move |c| {
            diesel::insert_into(task::table)
                .values(NewTask {
                    name: &task_json.name,
                })
                .get_result(c)
        })
        .await?;

    let response = NewTaskResponse { task: new_task };

    Ok(Created::new("/create").body(Json(response)))
}
