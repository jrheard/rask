use crate::db::DBConn;
use crate::db_queries;
use crate::form::{RecurrenceForm, TaskForm, WrappedNewRecurrenceTemplate, WrappedNewTask};
use crate::token::ApiToken;
use rask_lib::models::{RecurrenceTemplate, Task, MODE_COMPLETED};
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

// Tasks

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

    Ok(Created::new(format!("/task/{}", new_task.id)).body(Json(new_task)))
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
    db.run(move |conn| db_queries::uncomplete_task(conn, task_id))
        .await
        .map(|row| row.map(Json))
        .map_err(RaskApiError::DatabaseError)
}

#[post("/task/<task_id>/modify", data = "<task_form>")]
pub async fn modify_task(
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

// Recurrences

#[get("/recurrence/<recurrence_id>")]
pub async fn get_recurrence_by_id(
    db: DBConn,
    recurrence_id: i32,
    _token: ApiToken,
) -> Result<Option<Json<RecurrenceTemplate>>> {
    db.run(move |conn| db_queries::get_recurrence_by_id(conn, recurrence_id))
        .await
        .map(|row| row.map(Json))
        .map_err(RaskApiError::DatabaseError)
}

#[post("/recurrence", data = "<recurrence_form>")]
pub async fn create_recurrence(
    db: DBConn,
    recurrence_form: Form<RecurrenceForm>,
    _token: ApiToken,
) -> Result<Created<Json<RecurrenceTemplate>>> {
    let new_template = db
        .run(move |conn| {
            db_queries::create_recurrence(
                conn,
                WrappedNewRecurrenceTemplate::from(recurrence_form).0,
            )
        })
        .await?;

    Ok(Created::new(format!("/recurrence/{}", new_template.id)).body(Json(new_template)))
}

#[get("/recurrences/all")]
pub async fn get_recurrences(
    db: DBConn,
    _token: ApiToken,
) -> Result<Json<Vec<RecurrenceTemplate>>> {
    let recurrences = db
        .run(move |conn| db_queries::get_recurrences(conn))
        .await?;

    Ok(Json(recurrences))
}

#[post("/recurrence/<recurrence_id>/modify", data = "<recurrence_form>")]
pub async fn modify_recurrence(
    db: DBConn,
    recurrence_id: i32,
    recurrence_form: Form<RecurrenceForm>,
    _token: ApiToken,
) -> Result<Option<Json<RecurrenceTemplate>>> {
    db.run(move |conn| {
        db_queries::update_recurrence(
            conn,
            recurrence_id,
            WrappedNewRecurrenceTemplate::from(recurrence_form).0,
        )
    })
    .await
    .map(|row| row.map(Json))
    .map_err(RaskApiError::DatabaseError)
}

// Misc

#[get("/500")]
pub async fn return_500() -> RaskApiError {
    RaskApiError::IntentionalErrorForTesting
}

#[get("/healthcheck")]
pub async fn healthcheck() -> &'static str {
    "hello!"
}
