use crate::models::{NewTask, Task};
use crate::schema::task;
use diesel::prelude::*;
use diesel::PgConnection;

type Result<T, E = diesel::result::Error> = std::result::Result<T, E>;

pub fn get_tasks(conn: &PgConnection) -> Result<Vec<Task>> {
    task::table.load(conn)
}

pub fn get_task_by_id(conn: &PgConnection, task_id: i32) -> Result<Task> {
    task::table.filter(task::id.eq(task_id)).first(conn)
}

pub fn create_task(conn: &PgConnection, name: &str) -> Result<Task> {
    diesel::insert_into(task::table)
        .values(NewTask { name })
        .get_result(conn)
}
