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

pub fn assemble_rocket() -> Rocket<Build> {
    dotenv().ok();

    let db_url = env::var("RASK_DATABASE_URL").expect("RASK_DATABASE_URL must be defined");
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
                    endpoints::uncomplete_task,
                    endpoints::modify_task,
                    endpoints::healthcheck,
                    endpoints::create_recurrence,
                    endpoints::get_recurrence_by_id,
                    endpoints::get_recurrences,
                    endpoints::modify_recurrence
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
    use std::env;

    use super::assemble_rocket;
    use rocket::{http::Status, local::blocking::Client};

    const DATABASE_URL: &str = "postgres://postgres:password@localhost:5001/rask";

    #[test]
    fn test_500_response() {
        env::set_var("RASK_DATABASE_URL", DATABASE_URL);

        let client = Client::tracked(assemble_rocket()).unwrap();
        let response = client.get("/500").dispatch();

        assert_eq!(response.status(), Status::InternalServerError);
        assert_eq!(
            response.into_string(),
            Some("Error: Intentional error thrown for use in tests".to_string())
        );
    }
}
