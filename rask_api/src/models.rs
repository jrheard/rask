use diesel::Queryable;
use serde::Serialize;

#[derive(Queryable, Serialize)]
pub struct Task {
    pub id: i32,
    pub name: String,
}
