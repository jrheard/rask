use crate::args::{CompleteOpts, CreateOpts, InfoOpts, Opts, SubCommand};
use anyhow::{Context, Result};
use args::ModifyOpts;
use clap::Clap;
use rask_lib::models::{NewTask, Task};

pub mod args;

pub const DATE_FORMAT: &str = "%m/%d/%Y";

const API_ROOT: &str = "http://localhost";

/// Turns an `endpoint` like `task/1` into a full API URL.
fn make_url(endpoint: &str) -> String {
    let port = if std::env::var_os("RUST_TESTING").is_some() {
        "8002"
    } else {
        "8001"
    };

    format!("{}:{}/{}", API_ROOT, port, endpoint)
}

fn get_task(task_id: i32) -> Result<Task> {
    Ok(
        reqwest::blocking::get(make_url(&format!("task/{}", task_id)))
            .context("Unable to read task info from API")?
            .json::<Task>()?,
    )
}

fn complete_task(task_id: i32) -> Result<()> {
    let client = reqwest::blocking::Client::new();
    let task = client
        .post(make_url(&format!("task/{}/complete", task_id)))
        .send()?
        .error_for_status()
        .context("Unable to mark task completed")?
        .json::<Task>()?;

    println!("Completed task {}: '{}'", task.id, task.name);
    Ok(())
}

fn create_task(opts: CreateOpts) -> Result<()> {
    let client = reqwest::blocking::Client::new();
    let created_task = client
        .post(make_url("task"))
        .form(&NewTask::from(opts))
        .send()?
        .error_for_status()
        .context("Unable to create task")?
        .json::<Task>()?;

    println!("Successfully created task with ID {}.", created_task.id);
    Ok(())
}

fn task_info(task_id: i32) -> Result<()> {
    let task = get_task(task_id)?;

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
        .json::<Vec<Task>>()?;

    println!("Retrieved {} tasks", tasks.len());
    println!("======================");
    for task in tasks {
        println!("{}\t{}", task.id, task.name);
    }

    Ok(())
}

fn modify_task(opts: ModifyOpts) -> Result<()> {
    let task = get_task(opts.task_id)?;

    // For each Optional value in NewTask:
    // Set it to None if the user passed in the literal string "none",
    // otherwise fall back to `task`'s value for that field.
    let new_task_values = NewTask {
        name: opts.name.unwrap_or(task.name),
        project: match opts.project.as_deref() {
            Some("none") => None,
            _ => opts.project.or(task.project),
        },
        priority: match opts.priority.as_deref() {
            Some("none") => None,
            _ => opts.priority.or(task.priority),
        },
        due: match opts.due {
            None => task.due,
            Some(args::ParseDecision::Set(due)) => Some(due),
            Some(args::ParseDecision::Delete) => None,
        },
    };

    let client = reqwest::blocking::Client::new();
    let updated_task = client
        .post(make_url(&format!("task/{}/edit", task.id)))
        .form(&new_task_values)
        .send()?
        .error_for_status()
        .context("Unable to modify task")?
        .json::<Task>()?;

    println!("Updated task {}.", updated_task.id);

    Ok(())
}

pub fn run() -> Result<()> {
    let opts = Opts::parse();

    match opts.subcommand {
        SubCommand::Complete(CompleteOpts { task_id }) => complete_task(task_id),
        SubCommand::Create(create_opts) => create_task(create_opts),
        SubCommand::Info(InfoOpts { task_id }) => task_info(task_id),
        SubCommand::List(_) => list_tasks(),
        SubCommand::Modify(modify_opts) => modify_task(modify_opts),
    }
}
