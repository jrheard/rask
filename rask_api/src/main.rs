// TODO - can i get rid of the macro_use? i seem to need it for the table! usage in schema.rs
#[macro_use]
extern crate diesel;

use crate::schema::task;
use diesel::prelude::*;
use diesel::PgConnection;
use rocket::{get, launch, response::content::Json, routes};
use rocket_sync_db_pools::database;

pub mod models;
pub mod schema;

#[database("rask_db")]
pub struct DBConn(PgConnection);

#[get("/<task_id>")]
async fn hello(db: DBConn, task_id: i32) -> Option<Json<models::Task>> {
    db.run(move |conn| task::table.filter(task::id.eq(task_id)).first(conn))
        .await
        .map(Json)
        .ok()
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![hello])
        .attach(DBConn::fairing())
}
