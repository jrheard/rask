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

fn load_environment_variables() {
    if env::var("RUST_TESTING").is_ok() {
        dotenv::from_filename(".env.test").ok();
    } else {
        dotenv().ok();
    }
}

#[launch]
fn rocket() -> _ {
    load_environment_variables();

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

#[cfg(test)]
mod test {
    use super::rocket;
    use rocket::http::Status;
    use rocket::local::blocking::Client;

    #[test]
    fn test_get_tasks_when_no_tasks() {
        let client = Client::tracked(rocket()).unwrap();

        for url in ["/tasks/all", "/tasks/alive"] {
            let response = client.get(url).dispatch();
            assert_eq!(response.status(), Status::Ok);
            assert_eq!(response.into_string(), Some("{\"tasks\":[]}".to_string()));
        }
    }
}
