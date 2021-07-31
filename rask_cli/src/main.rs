use rask_lib::models;

const API_ROOT: &str = "http://localhost:8001";

/// Turns an `endpoint` like `task/1` into a full API URL.
fn make_url(endpoint: &str) -> String {
    format!("{}/{}", API_ROOT, endpoint)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tasks = reqwest::blocking::get(make_url("tasks/alive"))?.json::<Vec<models::Task>>()?;

    println!("Retrieved {} tasks", tasks.len());
    println!("======================");
    for task in tasks {
        println!("{}\t{}", task.id, task.name);
    }

    Ok(())
}
