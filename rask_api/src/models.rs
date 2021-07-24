use super::schema::task;
use diesel::Queryable;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug)]
pub struct Mode(pub &'static str);

pub const MODE_PENDING: Mode = Mode("pending");
pub const MODE_ACTIVE: Mode = Mode("active");
pub const MODE_COMPLETED: Mode = Mode("completed");
#[allow(dead_code)]
pub const MODE_DELETED: Mode = Mode("deleted");

#[derive(Queryable, Deserialize, Serialize, Identifiable, PartialEq, Eq, Debug)]
#[table_name = "task"]
pub struct Task {
    pub id: i32,
    pub name: String,
    pub mode: String,
}

#[derive(Insertable)]
#[table_name = "task"]
pub struct NewTask<'a> {
    pub name: &'a str,
}
