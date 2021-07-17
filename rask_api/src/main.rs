#[macro_use]
extern crate diesel;

use dotenv::dotenv;
use rocket::figment::util::map;
use rocket::figment::value::{Map, Value};
use rocket::{launch, routes};
use std::env;

mod db;
mod endpoints;
mod models;
mod schema;

#[launch]
fn rocket() -> _ {
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").unwrap();
    let db: Map<_, Value> = map! {
        "url" => db_url.into(),
        "pool_size" => 10.into()
    };

    rocket::custom(rocket::Config::figment().merge(("databases", map! {"rask_db" => db})))
        .mount("/", routes![endpoints::hello])
        .attach(db::DBConn::fairing())
}
