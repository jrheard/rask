use crate::args::{CompleteOpts, CreateOpts, InfoOpts, Opts, SubCommand};
use anyhow::{Context, Result};
use clap::Clap;
use rask_lib::models;

pub mod args;

pub const DATE_FORMAT: &str = "%m/%d/%Y";

const API_ROOT: &str = "http://localhost:8001";

/// Turns an `endpoint` like `task/1` into a full API URL.
fn make_url(endpoint: &str) -> String {
    format!("{}/{}", API_ROOT, endpoint)
}

fn complete_task(task_id: i32) -> Result<()> {
    println!("Marking task {} as completed...", task_id);

    let client = reqwest::blocking::Client::new();
    let result = client
        .post(make_url(&format!("task/{}/complete", task_id)))
        .send()?
        .error_for_status()
        .context("Unable to mark task completed")
        .map(|_| ());

    println!("Success!");

    result
}

fn create_task(opts: CreateOpts) -> Result<()> {
    let client = reqwest::blocking::Client::new();
    let created_task = client
        .post(make_url("task"))
        .form(&models::NewTask::from(opts))
        .send()?
        .error_for_status()
        .context("Unable to mark task completed")?
        .json::<models::Task>()?;

    println!("Successfully created task with ID {}.", created_task.id);
    Ok(())
}

fn task_info(task_id: i32) -> Result<()> {
    let task = reqwest::blocking::get(make_url(&format!("task/{}", task_id)))
        .context("Unable to read task info from API")?
        .json::<models::Task>()?;

    println!("Task {}:", task.id);
    println!("==============================");

    println!("Name:\t\t{}", task.name);
    println!(
        "Project:\t{}",
        task.project.unwrap_or_else(|| "N/A".to_string())
    );
    println!(
        "Priority:\t{}",
        task.priority.unwrap_or_else(|| "N/A".to_string())
    );
    println!(
        "Due:\t\t{}",
        task.due
            .map(|due| due.format(DATE_FORMAT).to_string())
            .unwrap_or_else(|| "N/A".to_string())
    );

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

pub fn run() -> Result<()> {
    let opts = Opts::parse();

    match opts.subcommand {
        SubCommand::Complete(CompleteOpts { task_id }) => complete_task(task_id),
        SubCommand::Create(create_opts) => create_task(create_opts),
        SubCommand::Info(InfoOpts { task_id }) => task_info(task_id),
        SubCommand::List(_) => list_tasks(),
    }
}
