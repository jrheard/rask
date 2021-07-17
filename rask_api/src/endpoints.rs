use crate::db::DBConn;
use crate::models::Task;
use crate::schema::task;
use diesel::prelude::*;
use rocket::get;
use rocket::serde::json::Json;

#[get("/<task_id>")]
pub async fn hello(db: DBConn, task_id: i32) -> Option<Json<Task>> {
    db.run(move |conn| task::table.filter(task::id.eq(task_id)).first(conn))
        .await
        .map(Json)
        .ok()
}
