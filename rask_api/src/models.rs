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

#[derive(Clone, Copy, Debug)]
pub struct Priority(pub &'static str);

pub const PRIORITY_HIGH: Priority = Priority("H");
pub const PRIORITY_MEDIUM: Priority = Priority("M");
pub const PRIORITY_LOW: Priority = Priority("L");

#[derive(Queryable, Deserialize, Serialize, Identifiable, PartialEq, Eq, Debug, Clone)]
#[table_name = "task"]
pub struct Task {
    pub id: i32,
    pub name: String,
    pub project: Option<String>,
    pub priority: Option<String>,
    pub mode: String,
    pub due: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable, Serialize)]
#[table_name = "task"]
pub struct NewTask {
    pub name: String,
    pub project: Option<String>,
    pub priority: Option<String>,
}
