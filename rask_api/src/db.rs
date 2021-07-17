use diesel::PgConnection;
use rocket_sync_db_pools::database;

#[database("rask_db")]
pub struct DBConn(PgConnection);
