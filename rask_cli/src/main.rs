use anyhow::{Context, Result};
use clap::Clap;
use rask_lib::models;

const API_ROOT: &str = "http://localhost:8001";

/// Turns an `endpoint` like `task/1` into a full API URL.
fn make_url(endpoint: &str) -> String {
    format!("{}/{}", API_ROOT, endpoint)
}

#[derive(Clap)]
struct Opts {
    #[clap(subcommand)]
    subcommand: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    Complete(Complete),
    List(List),
}

#[derive(Clap)]
struct List {}
#[derive(Clap)]
struct Complete {
    task_id: i32,
}

fn complete_task(task_id: i32) -> Result<()> {
    Ok(())
}

fn list_tasks() -> Result<()> {
    let tasks = reqwest::blocking::get(make_url("tasks/alive"))
        .context("Unable to read alive tasks from API")?
        .json::<Vec<models::Task>>()?;

    println!("Retrieved {} tasks", tasks.len());
    println!("======================");
    for task in tasks {
        println!("{}\t{}", task.id, task.name);
    }

    Ok(())
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    match opts.subcommand {
        SubCommand::Complete(Complete { task_id }) => complete_task(task_id),
        SubCommand::List(_) => list_tasks(),
    }
}
