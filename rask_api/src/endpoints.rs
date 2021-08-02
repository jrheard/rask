use crate::db::DBConn;
use crate::db_queries;
use crate::form::{TaskForm, WrappedNewTask};
use rask_lib::models::{Task, MODE_COMPLETED};
use rocket::form::Form;
use rocket::http::{ContentType, Status};
use rocket::outcome::Outcome;
use rocket::request::{self, FromRequest};
use rocket::response::status::Created;
use rocket::response::{Responder, Response};
use rocket::serde::json::Json;
use rocket::{get, post, Request};
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

struct ApiToken;

#[derive(Debug)]
enum ApiTokenError {
    NoHeader,
    MalformedHeader,
    InvalidToken,
    DatabaseError,
}

fn parse_auth_header(header: &str) -> Option<&str> {
    let split_header = header.split(' ').collect::<Vec<_>>();

    if split_header.len() != 2 || split_header[0] != "Bearer" {
        return None;
    }

    Some(split_header[1])
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiToken {
    type Error = ApiTokenError;

    // TODO rewrite
    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let auth_header = req.headers().get_one("Authorization");

        if auth_header.is_none() {
            return Outcome::Failure((Status::BadRequest, ApiTokenError::NoHeader));
        }

        let parsed_header = parse_auth_header(auth_header.unwrap());
        if parsed_header.is_none() {
            return Outcome::Failure((Status::BadRequest, ApiTokenError::MalformedHeader));
        }

        let token = parsed_header.unwrap().to_string();

        if let Outcome::Success(db) = req.guard::<DBConn>().await {
            let token_row = db
                .run(move |conn| db_queries::token_exists(conn, &token))
                .await;

            match token_row {
                Ok(true) => Outcome::Success(ApiToken),
                Ok(false) => Outcome::Failure((Status::Unauthorized, ApiTokenError::InvalidToken)),
                Err(_) => {
                    Outcome::Failure((Status::InternalServerError, ApiTokenError::DatabaseError))
                }
            }
        } else {
            Outcome::Failure((Status::InternalServerError, ApiTokenError::DatabaseError))
        }
    }
}

type Result<T, E = RaskApiError> = std::result::Result<T, E>;

#[get("/task/<task_id>")]
pub async fn get_task_by_id(db: DBConn, task_id: i32) -> Result<Option<Json<Task>>> {
    db.run(move |conn| db_queries::get_task_by_id(conn, task_id))
        .await
        .map(|row| row.map(Json))
        .map_err(RaskApiError::DatabaseError)
}

#[get("/tasks/all")]
pub async fn get_tasks(db: DBConn) -> Result<Json<Vec<Task>>> {
    let tasks = db.run(move |conn| db_queries::get_tasks(conn)).await?;

    Ok(Json(tasks))
}

#[get("/tasks/alive")]
pub async fn get_alive_tasks(db: DBConn) -> Result<Json<Vec<Task>>> {
    let tasks = db
        .run(move |conn| db_queries::get_alive_tasks(conn))
        .await?;

    Ok(Json(tasks))
}

#[post("/task", data = "<task_form>")]
pub async fn create_task(db: DBConn, task_form: Form<TaskForm>) -> Result<Created<Json<Task>>> {
    let new_task = db
        .run(move |conn| db_queries::create_task(conn, WrappedNewTask::from(task_form).0))
        .await?;

    Ok(Created::new("/task").body(Json(new_task)))
}

#[post("/task/<task_id>/complete")]
pub async fn complete_task(db: DBConn, task_id: i32) -> Result<Option<Json<Task>>> {
    db.run(move |conn| db_queries::update_mode(conn, task_id, MODE_COMPLETED))
        .await
        .map(|row| row.map(Json))
        .map_err(RaskApiError::DatabaseError)
}

#[post("/task/<task_id>/edit", data = "<task_form>")]
pub async fn edit_task(
    db: DBConn,
    task_id: i32,
    task_form: Form<TaskForm>,
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
