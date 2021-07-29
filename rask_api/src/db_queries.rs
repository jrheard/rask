use rask_lib::models::{Mode, NewTask, Task, MODE_ACTIVE, MODE_PENDING};
use rask_lib::schema::task;
use diesel::dsl::any;
use diesel::prelude::*;
use diesel::PgConnection;

pub fn get_tasks(conn: &PgConnection) -> QueryResult<Vec<Task>> {
    task::table.order(task::id).load(conn)
}

pub fn get_alive_tasks(conn: &PgConnection) -> QueryResult<Vec<Task>> {
    task::table
        .filter(task::mode.eq(any(vec![MODE_PENDING.0, MODE_ACTIVE.0])))
        .order(task::id)
        .load(conn)
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

pub fn create_task(conn: &PgConnection, new_task: NewTask) -> QueryResult<Task> {
    diesel::insert_into(task::table)
        .values(new_task)
        .get_result(conn)
}

pub fn update_task(
    conn: &PgConnection,
    task_id: i32,
    updated_fields: NewTask,
) -> QueryResult<Option<Task>> {
    diesel::update(task::table.find(task_id))
        .set(updated_fields)
        .get_result(conn)
        .optional()
}
