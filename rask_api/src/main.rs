#[macro_use]
extern crate diesel;

use crate::schema::task;
use diesel::prelude::*;
use diesel::PgConnection;
use dotenv::dotenv;
use rocket::figment::util::map;
use rocket::figment::value::{Map, Value};
use rocket::serde::json::Json;
use rocket::{get, launch, routes};
use rocket_sync_db_pools::database;
use std::env;

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
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").unwrap();
    let db: Map<_, Value> = map! {
        "url" => db_url.into(),
        "pool_size" => 10.into()
    };

    rocket::custom(rocket::Config::figment().merge(("databases", map!["rask_db" => db])))
        .mount("/", routes![hello])
        .attach(DBConn::fairing())
}
