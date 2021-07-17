use super::schema::task;
use diesel::Queryable;
use serde::Serialize;

#[derive(Queryable, Serialize)]
pub struct Task {
    pub id: i32,
    pub name: String,
}

#[derive(Insertable)]
#[table_name = "task"]
pub struct NewTask<'a> {
    pub name: &'a str,
}
