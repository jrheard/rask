use crate::args::{CompleteOpts, CreateOpts, InfoOpts, Opts, SubCommand};
use anyhow::{Context, Result};
use args::ModifyOpts;
use clap::Clap;
use rask_lib::models::{NewTask, Task};
use reqwest::blocking::{Client, RequestBuilder};
use reqwest::header::AUTHORIZATION;
use std::env;

pub mod args;

pub const DATE_FORMAT: &str = "%m/%d/%Y";

/// Turns an `endpoint` like `task/1` into a full API URL.
fn make_url(endpoint: &str) -> String {
    let root = env::var("RASK_API_ROOT").unwrap_or_else(|_| "https://rask.jrheard.com".to_string());

    format!("{}/{}", root, endpoint)
}

trait Authorizable {
    fn add_authorization_header(self, token: &str) -> Self;
}

impl Authorizable for RequestBuilder {
    fn add_authorization_header(self, token: &str) -> Self {
        self.header(AUTHORIZATION, format!("Bearer {}", token))
    }
}

fn get_task(task_id: i32, token: &str) -> Result<Task> {
    let client = Client::new();
    Ok(client
        .get(make_url(&format!("task/{}", task_id)))
        .add_authorization_header(token)
        .send()
        .context("Unable to read task info from API")?
        .json::<Task>()?)
}

fn complete_task(task_id: i32, token: &str) -> Result<()> {
    let client = Client::new();
    let task = client
        .post(make_url(&format!("task/{}/complete", task_id)))
        .add_authorization_header(token)
        .send()?
        .error_for_status()
        .context("Unable to mark task completed")?
        .json::<Task>()?;

    println!("Completed task {}: '{}'", task.id, task.name);
    Ok(())
}

fn create_task(opts: CreateOpts, token: &str) -> Result<()> {
    let client = Client::new();
    let created_task = client
        .post(make_url("task"))
        .add_authorization_header(token)
        .form(&NewTask::from(opts))
        .send()?
        .error_for_status()
        .context("Unable to create task")?
        .json::<Task>()?;

    println!("Successfully created task with ID {}.", created_task.id);
    Ok(())
}

fn task_info(task_id: i32, token: &str) -> Result<()> {
    let task = get_task(task_id, token)?;

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

fn list_tasks(token: &str) -> Result<()> {
    let client = Client::new();
    let tasks = client
        .get(make_url("tasks/alive"))
        .add_authorization_header(token)
        .send()
        .context("Unable to read alive tasks from API")?
        .json::<Vec<Task>>()?;

    println!("Retrieved {} tasks", tasks.len());
    println!("======================");
    for task in tasks {
        println!("{}\t{}", task.id, task.name);
    }

    Ok(())
}

fn modify_task(opts: ModifyOpts, token: &str) -> Result<()> {
    let task = get_task(opts.task_id, token)?;

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

    let client = Client::new();
    let updated_task = client
        .post(make_url(&format!("task/{}/edit", task.id)))
        .add_authorization_header(token)
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
    let token = env::var("RASK_API_TOKEN").expect("No value found for RASK_API_TOKEN");

    match opts.subcommand {
        SubCommand::Complete(CompleteOpts { task_id }) => complete_task(task_id, &token),
        SubCommand::Create(create_opts) => create_task(create_opts, &token),
        SubCommand::Info(InfoOpts { task_id }) => task_info(task_id, &token),
        SubCommand::List(_) => list_tasks(&token),
        SubCommand::Modify(modify_opts) => modify_task(modify_opts, &token),
    }
}
