use crate::models::{NewTask, Task};
use crate::schema::task;
use diesel::prelude::*;
use diesel::PgConnection;

pub fn get_tasks(conn: &PgConnection) -> QueryResult<Vec<Task>> {
    task::table.load(conn)
}

pub fn get_task_by_id(conn: &PgConnection, task_id: i32) -> QueryResult<Option<Task>> {
    task::table
        .filter(task::id.eq(task_id))
        .first(conn)
        .optional()
}

pub fn create_task(conn: &PgConnection, name: &str) -> QueryResult<Task> {
    diesel::insert_into(task::table)
        .values(NewTask { name })
        .get_result(conn)
}
