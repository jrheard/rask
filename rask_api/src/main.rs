#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use crate::db::DBConn;
use dotenv::dotenv;
use rocket::fairing::AdHoc;
use rocket::figment::util::map;
use rocket::figment::value::{Map, Value};
use rocket::{launch, routes, Build, Rocket};
use std::env;

mod db;
mod db_queries;
mod endpoints;
mod models;
mod schema;

async fn run_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
    embed_migrations!();

    let conn = DBConn::get_one(&rocket).await.expect("database connection");
    conn.run(|c| embedded_migrations::run(c))
        .await
        .expect("diesel migrations");

    rocket
}

#[launch]
fn rocket() -> _ {
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be defined");
    let db: Map<_, Value> = map! {
        "url" => db_url.into(),
        "pool_size" => 10.into()
    };

    rocket::custom(rocket::Config::figment().merge(("databases", map! {"rask_db" => db})))
        .mount(
            "/",
            routes![
                endpoints::get_tasks,
                endpoints::get_alive_tasks,
                endpoints::get_task_by_id,
                endpoints::create_task,
                endpoints::complete_task
            ],
        )
        .attach(DBConn::fairing())
        .attach(AdHoc::on_ignite("Diesel Migrations", run_migrations))
}
