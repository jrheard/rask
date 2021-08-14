use crate::db::DBConn;
use crate::db_queries;
use crate::form::{TaskForm, WrappedNewTask};
use crate::token::ApiToken;
use rask_lib::models::{Task, MODE_COMPLETED, MODE_DELETED};
use rocket::form::Form;
use rocket::http::{ContentType, Status};
use rocket::response::status::Created;
use rocket::response::{Responder, Response};
use rocket::serde::json::Json;
use rocket::{get, post};
use std::io::Cursor;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RaskApiError {
    #[error(transparent)]
    DatabaseError(#[from] diesel::result::Error),

    #[error("Intentional error thrown for use in tests")]
    IntentionalErrorForTesting,
}

impl<'r> Responder<'r, 'static> for RaskApiError {
    /// Respond with a 500 status code.
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
pub async fn get_task_by_id(
    db: DBConn,
    task_id: i32,
    _token: ApiToken,
) -> Result<Option<Json<Task>>> {
    db.run(move |conn| db_queries::get_task_by_id(conn, task_id, true))
        .await
        .map(|row| row.map(Json))
        .map_err(RaskApiError::DatabaseError)
}

#[get("/tasks/all")]
pub async fn get_tasks(db: DBConn, _token: ApiToken) -> Result<Json<Vec<Task>>> {
    let tasks = db.run(move |conn| db_queries::get_tasks(conn)).await?;

    Ok(Json(tasks))
}

#[get("/tasks/alive")]
pub async fn get_alive_tasks(db: DBConn, _token: ApiToken) -> Result<Json<Vec<Task>>> {
    let tasks = db
        .run(move |conn| db_queries::get_alive_tasks(conn))
        .await?;

    Ok(Json(tasks))
}

#[post("/task", data = "<task_form>")]
pub async fn create_task(
    db: DBConn,
    task_form: Form<TaskForm>,
    _token: ApiToken,
) -> Result<Created<Json<Task>>> {
    let new_task = db
        .run(move |conn| db_queries::create_task(conn, WrappedNewTask::from(task_form).0))
        .await?;

    Ok(Created::new("/task").body(Json(new_task)))
}

#[post("/task/<task_id>/complete")]
pub async fn complete_task(
    db: DBConn,
    task_id: i32,
    _token: ApiToken,
) -> Result<Option<Json<Task>>> {
    db.run(move |conn| db_queries::update_mode(conn, task_id, MODE_COMPLETED))
        .await
        .map(|row| row.map(Json))
        .map_err(RaskApiError::DatabaseError)
}

#[post("/task/<task_id>/uncomplete")]
pub async fn uncomplete_task(
    db: DBConn,
    task_id: i32,
    _token: ApiToken,
) -> Result<Option<Json<Task>>> {
    let result = db
        .run(move |conn| db_queries::get_task_by_id(conn, task_id, false))
        .await
        .map_err(RaskApiError::DatabaseError)?;

    if let Some(task) = result {
        if task.mode == MODE_COMPLETED.0 {
            db.run(move |conn| db_queries::update_mode(conn, task_id, MODE_COMPLETED))
                .await
                .map(|o| o.map(Json))
                .map_err(RaskApiError::DatabaseError)
        } else {
            Ok(Some(Json(task)))
        }
    } else {
        Ok(None)
    }
}

#[post("/task/<task_id>/edit", data = "<task_form>")]
pub async fn edit_task(
    db: DBConn,
    task_id: i32,
    task_form: Form<TaskForm>,
    _token: ApiToken,
) -> Result<Option<Json<Task>>> {
    db.run(move |conn| db_queries::update_task(conn, task_id, WrappedNewTask::from(task_form).0))
        .await
        .map(|row| row.map(Json))
        .map_err(RaskApiError::DatabaseError)
}

#[get("/500")]
pub async fn return_500() -> RaskApiError {
    RaskApiError::IntentionalErrorForTesting
}

#[get("/healthcheck")]
pub async fn healthcheck() -> &'static str {
    "hello!"
}
