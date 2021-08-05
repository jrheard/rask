#[macro_use]
extern crate diesel_migrations;

use crate::db::DBConn;
use dotenv::dotenv;
use rocket::fairing::AdHoc;
use rocket::figment::util::map;
use rocket::figment::value::{Map, Value};
use rocket::{routes, Build, Rocket};
use std::env;

mod db;
mod db_queries;
pub mod endpoints;
mod form;
mod token;

/// Runs Diesel migrations as part of `rocket`'s initialization.
async fn run_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
    embed_migrations!("../rask_lib/migrations");

    let conn = DBConn::get_one(&rocket).await.expect("database connection");
    conn.run(|c| embedded_migrations::run(c))
        .await
        .expect("diesel migrations");

    rocket
}

/// Loads environment variables from .env files.
pub fn load_environment_variables() {
    if env::var("RUST_TESTING").is_ok() {
        dotenv::from_filename(".env.test").ok();
    }
    dotenv().ok();
}

pub fn assemble_rocket() -> Rocket<Build> {
    load_environment_variables();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be defined");
    println!("db url is {}", db_url);
    let db: Map<_, Value> = map! {
        "url" => db_url.into(),
        "pool_size" => 10.into()
    };

    let mut r =
        rocket::custom(rocket::Config::figment().merge(("databases", map! {"rask_db" => db})))
            .mount(
                "/",
                routes![
                    endpoints::get_tasks,
                    endpoints::get_alive_tasks,
                    endpoints::get_task_by_id,
                    endpoints::create_task,
                    endpoints::complete_task,
                    endpoints::edit_task,
                    endpoints::healthcheck,
                ],
            )
            .attach(DBConn::fairing())
            .attach(AdHoc::on_ignite("Diesel Migrations", run_migrations));

    if cfg!(test) {
        // This endpoint is only used for testing our 500 response codepath.
        r = r.mount("/", routes![endpoints::return_500]);
    }

    r
}

#[cfg(test)]
mod tests {
    use super::assemble_rocket;
    use rocket::{http::Status, local::blocking::Client};

    #[test]
    fn test_500_response() {
        let client = Client::tracked(assemble_rocket()).unwrap();
        let response = client.get("/500").dispatch();

        assert_eq!(response.status(), Status::InternalServerError);
        assert_eq!(
            response.into_string(),
            Some("Error: Intentional error thrown for use in tests".to_string())
        );
    }
}
