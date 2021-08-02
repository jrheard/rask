use crate::schema::{api_token, task};
use diesel::prelude::*;
use std::panic;

/// Deletes all rows in the `task` table.
fn delete_all_tasks(conn: &PgConnection) {
    diesel::delete(task::table).execute(conn).unwrap();
}

/// Deletes all rows in the `api_token` table.
fn delete_all_tokens(conn: &PgConnection) {
    diesel::delete(api_token::table).execute(conn).unwrap();
}

/// Runs a chunk of test code in a setup/teardown block.
/// Via https://medium.com/@ericdreichert/test-setup-and-teardown-in-rust-without-a-framework-ba32d97aa5ab .
pub fn run_test<T>(test: T, conn: PgConnection)
where
    T: FnOnce() + panic::UnwindSafe,
{
    let result = panic::catch_unwind(|| test());

    delete_all_tasks(&conn);
    delete_all_tokens(&conn);

    assert!(result.is_ok());
}
