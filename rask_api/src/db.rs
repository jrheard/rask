use rocket_sync_db_pools::database;

#[database("rask_db")]
pub struct DBConn(diesel::PgConnection);
