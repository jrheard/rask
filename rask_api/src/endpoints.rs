use crate::db::DBConn;
use crate::db_queries;
use crate::models::Task;
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
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        let status = match &self {
            Self::DatabaseError(err) => match err {
                diesel::result::Error::NotFound => Status::NotFound,
                _ => Status::InternalServerError,
            },
        };

        let body = format!("Error: {}", self);
        let response = Response::build()
            .status(status)
            .header(ContentType::Plain)
            .sized_body(body.len(), Cursor::new(body))
            .finalize();

        Ok(response)
    }
}

type Result<T, E = RaskApiError> = std::result::Result<T, E>;

#[get("/task/<task_id>")]
pub async fn get_task_by_id(db: DBConn, task_id: i32) -> Result<Json<Task>> {
    db.run(move |conn| db_queries::get_task_by_id(conn, task_id))
        .await
        .map(Json)
        .map_err(RaskApiError::DatabaseError)
}

#[derive(Serialize)]
pub struct TaskListResponse {
    tasks: Vec<Task>,
}

#[get("/tasks")]
pub async fn get_tasks(db: DBConn) -> Result<Json<TaskListResponse>> {
    let tasks = db.run(move |conn| db_queries::get_tasks(conn)).await?;

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
        .run(move |conn| db_queries::create_task(conn, &task_json.name))
        .await?;

    let response = NewTaskResponse { task: new_task };

    Ok(Created::new("/task").body(Json(response)))
}
