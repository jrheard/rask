use diesel::dsl::{any, exists};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::select;
use diesel::sql_types::Bool;
use diesel::PgConnection;
use rask_lib::models::{
    Mode, NewTask, Task, MODE_ACTIVE, MODE_COMPLETED, MODE_DELETED, MODE_PENDING,
};
use rask_lib::schema::api_token;
use rask_lib::schema::task;

type SqlExpr<'a, Table, SqlType> = Box<dyn BoxableExpression<Table, Pg, SqlType = SqlType> + 'a>;

pub fn alive_tasks<'a>() -> SqlExpr<'a, task::table, Bool> {
    Box::new(task::mode.eq(any(vec![MODE_PENDING.0, MODE_ACTIVE.0])))
}

pub fn get_tasks(conn: &PgConnection) -> QueryResult<Vec<Task>> {
    task::table.order(task::id).load(conn)
}

pub fn get_alive_tasks(conn: &PgConnection) -> QueryResult<Vec<Task>> {
    task::table.filter(alive_tasks()).order(task::id).load(conn)
}

pub fn get_task_by_id(
    conn: &PgConnection,
    task_id: i32,
    include_deleted: bool,
) -> QueryResult<Option<Task>> {
    let mut query = task::table.find(task_id).into_boxed();

    if !include_deleted {
        query = query.filter(task::mode.ne(MODE_DELETED.0));
    }

    query.first(conn).optional()
}

pub fn update_mode(conn: &PgConnection, task_id: i32, mode: Mode) -> QueryResult<Option<Task>> {
    diesel::update(task::table.find(task_id))
        .set(task::mode.eq(mode.0))
        .get_result(conn)
        .optional()
}

pub fn uncomplete_task(conn: &PgConnection, task_id: i32) -> QueryResult<Option<Task>> {
    let result = get_task_by_id(conn, task_id, false)?;

    match result {
        Some(task) if task.mode == MODE_COMPLETED.0 => update_mode(conn, task_id, MODE_PENDING),
        x => Ok(x),
    }
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

pub fn token_exists(conn: &PgConnection, token: &str) -> QueryResult<bool> {
    select(exists(api_token::table.find(token))).get_result(conn)
}
