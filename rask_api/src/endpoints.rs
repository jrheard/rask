use crate::db::DBConn;
use crate::models::{NewTask, Task};
use crate::schema::task;
use diesel::prelude::*;
use rocket::response::status::Created;
use rocket::serde::json::Json;
use rocket::{get, post};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct TaskJSON {
    name: String,
}

#[derive(Serialize)]
pub struct TaskListResponse {
    tasks: Vec<Task>,
}

#[derive(Serialize)]
pub struct NewTaskResponse {
    task: Task,
}

#[get("/task/<task_id>")]
pub async fn get_task_by_id(db: DBConn, task_id: i32) -> Option<Json<Task>> {
    db.run(move |conn| task::table.filter(task::id.eq(task_id)).first(conn))
        .await
        .map(Json)
        .ok()
}

#[get("/tasks")]
pub async fn get_tasks(db: DBConn) -> Json<TaskListResponse> {
    let tasks = db.run(move |conn| task::table.load(conn)).await.unwrap();

    Json(TaskListResponse { tasks })
}

#[post("/task", format = "json", data = "<task_json>")]
// TODO return result
pub async fn create_task(db: DBConn, task_json: Json<TaskJSON>) -> Created<Json<NewTaskResponse>> {
    let new_task = db
        .run(move |c| {
            diesel::insert_into(task::table)
                .values(NewTask {
                    name: &task_json.name,
                })
                .get_result(c)
        })
        .await
        .unwrap();

    let response = NewTaskResponse { task: new_task };

    Created::new("/create").body(Json(response))
}
