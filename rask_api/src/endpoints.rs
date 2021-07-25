use crate::db::DBConn;
use crate::db_queries;
use crate::models;
use crate::models::{NewTask, Task, MODE_COMPLETED};
use rocket::form;
use rocket::form::{Form, FromForm, FromFormField};
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct TaskListResponse {
    pub tasks: Vec<Task>,
}

#[derive(Deserialize, Serialize)]
pub struct NewTaskResponse {
    pub task: Task,
}

#[get("/task/<task_id>")]
pub async fn get_task_by_id(db: DBConn, task_id: i32) -> Result<Option<Json<Task>>> {
    db.run(move |conn| db_queries::get_task_by_id(conn, task_id))
        .await
        .map(|row| row.map(Json))
        .map_err(RaskApiError::DatabaseError)
}

#[get("/tasks/all")]
pub async fn get_tasks(db: DBConn) -> Result<Json<TaskListResponse>> {
    let tasks = db.run(move |conn| db_queries::get_tasks(conn)).await?;

    Ok(Json(TaskListResponse { tasks }))
}

#[get("/tasks/alive")]
pub async fn get_alive_tasks(db: DBConn) -> Result<Json<TaskListResponse>> {
    let tasks = db
        .run(move |conn| db_queries::get_alive_tasks(conn))
        .await?;

    Ok(Json(TaskListResponse { tasks }))
}

/// Task projects must be a single word or None.
fn validate_project<'v>(project: &Option<String>) -> form::Result<'v, ()> {
    match project.as_deref() {
        Some(project) if project.split(' ').count() != 1 => {
            Err(form::Error::validation("project must be a single word or blank").into())
        }
        _ => Ok(()),
    }
}

#[derive(FromFormField)]
pub enum Priority {
    #[field(value = "H")]
    High,
    #[field(value = "M")]
    Medium,
    #[field(value = "L")]
    Low,
}

impl From<Priority> for String {
    fn from(form_priority: Priority) -> Self {
        match form_priority {
            Priority::High => models::PRIORITY_HIGH.0,
            Priority::Medium => models::PRIORITY_MEDIUM.0,
            Priority::Low => models::PRIORITY_LOW.0,
        }
        .to_string()
    }
}

#[derive(FromForm)]
pub struct TaskForm {
    name: String,
    #[field(validate=validate_project())]
    project: Option<String>,
    priority: Option<Priority>,
}

impl From<Form<TaskForm>> for NewTask {
    fn from(form: Form<TaskForm>) -> Self {
        let form = form.into_inner();
        NewTask {
            name: form.name,
            project: form.project,
            priority: form.priority.map(|priority| priority.into()),
        }
    }
}

#[post("/task", data = "<task_form>")]
pub async fn create_task(
    db: DBConn,
    task_form: Form<TaskForm>,
) -> Result<Created<Json<NewTaskResponse>>> {
    let new_task = db
        .run(move |conn| db_queries::create_task(conn, task_form.into()))
        .await?;

    let response = NewTaskResponse { task: new_task };

    Ok(Created::new("/task").body(Json(response)))
}

#[post("/task/<task_id>/complete")]
pub async fn complete_task(db: DBConn, task_id: i32) -> Result<Option<Json<Task>>> {
    db.run(move |conn| db_queries::update_mode(conn, task_id, MODE_COMPLETED))
        .await
        .map(|row| row.map(Json))
        .map_err(RaskApiError::DatabaseError)
}
