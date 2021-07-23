use crate::models::{Mode, NewTask, Task};
use crate::schema::task;
use diesel::prelude::*;
use diesel::PgConnection;

pub fn get_tasks(conn: &PgConnection) -> QueryResult<Vec<Task>> {
    task::table.order(task::id).load(conn)
}

pub fn get_task_by_id(conn: &PgConnection, task_id: i32) -> QueryResult<Option<Task>> {
    task::table.find(task_id).first(conn).optional()
}

pub fn update_mode(conn: &PgConnection, task_id: i32, mode: Mode) -> QueryResult<Option<Task>> {
    diesel::update(task::table.find(task_id))
        .set(task::mode.eq(mode.0))
        .get_result(conn)
        .optional()
}

pub fn create_task(conn: &PgConnection, name: &str) -> QueryResult<Task> {
    diesel::insert_into(task::table)
        .values(NewTask { name })
        .get_result(conn)
}
