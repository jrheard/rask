use crate::args::{
    CompleteOpts, CreateOpts, InfoOpts, ListOpts, Opts, RecurSubCommand, SubCommand, UncompleteOpts,
};
use anyhow::{Context, Result};
use args::{CreateRecurrenceOpts, ModifyOpts};
use clap::Clap;
use rask_lib::models::{NewRecurrenceTemplate, NewTask, RecurrenceTemplate, Task};
use reqwest::blocking::{Client, RequestBuilder, Response};
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

enum Method {
    Get,
    Post,
}

fn make_request<T>(method: Method, url: String, form: Option<T>) -> reqwest::Result<Response>
where
    T: serde::Serialize,
{
    let token = env::var("RASK_API_TOKEN").expect("No value found for RASK_API_TOKEN");

    let client = Client::new();

    let mut builder = match method {
        Method::Get => client.get(url),
        Method::Post => client.post(url),
    }
    .add_authorization_header(&token);

    if let Some(form) = form {
        builder = builder.form(&form);
    }

    builder.send()?.error_for_status()
}

// Tasks

fn print_task(task: &Task) {
    println!("==========");
    println!("Task {}:", task.id);
    println!("==========");
    println!("Name:\t\t{}", task.name);
    println!("Mode:\t\t{}", task.mode);
    println!("Created:\t{}", task.time_created);
    println!("Project:\t{}", task.project.as_deref().unwrap_or("N/A"));
    println!("Priority:\t{}", task.priority.as_deref().unwrap_or("N/A"));
    println!(
        "Due:\t\t{}",
        task.due
            .map(|due| due.format(DATE_FORMAT).to_string())
            .unwrap_or_else(|| "N/A".to_string())
    );
}

fn get_task(task_id: i32) -> Result<Task> {
    Ok(
        make_request::<NewTask>(Method::Get, make_url(&format!("task/{}", task_id)), None)
            .context("Unable to read task info from API")?
            .json::<Task>()?,
    )
}

fn complete_task(task_id: i32) -> Result<()> {
    let task = make_request::<NewTask>(
        Method::Post,
        make_url(&format!("task/{}/complete", task_id)),
        None,
    )
    .context("Unable to mark task completed")?
    .json::<Task>()?;

    println!("Completed task.");
    print_task(&task);
    Ok(())
}

fn uncomplete_task(task_id: i32) -> Result<()> {
    let task = make_request::<NewTask>(
        Method::Post,
        make_url(&format!("task/{}/uncomplete", task_id)),
        None,
    )
    .context("Unable to mark task uncompleted")?
    .json::<Task>()?;

    println!("Uncompleted task.");
    print_task(&task);
    Ok(())
}

fn create_task(opts: CreateOpts) -> Result<()> {
    let created_task =
        make_request::<NewTask>(Method::Post, make_url("task"), Some(NewTask::from(opts)))
            .context("Unable to create task")?
            .json::<Task>()?;

    println!("Successfully created task.");
    print_task(&created_task);
    Ok(())
}

fn task_info(task_id: i32) -> Result<()> {
    let task = get_task(task_id)?;
    print_task(&task);
    Ok(())
}

fn list_tasks(include_all_tasks: bool) -> Result<()> {
    let endpoint = if include_all_tasks {
        "tasks/all"
    } else {
        "tasks/alive"
    };

    let tasks = make_request::<NewTask>(Method::Get, make_url(endpoint), None)
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

    let updated_task = make_request(
        Method::Post,
        make_url(&format!("task/{}/edit", task.id)),
        Some(new_task_values),
    )
    .context("Unable to modify task")?
    .json::<Task>()?;

    println!("Updated task.");
    print_task(&updated_task);

    Ok(())
}

// Recurrences

fn print_recurrence(recurrence: &RecurrenceTemplate) {
    println!("==========");
    println!("Recurrence {}:", recurrence.id);
    println!("==========");
    println!("Name:\t\t{}", recurrence.name);
    println!("Created:\t{}", recurrence.time_created);
    println!(
        "Project:\t{}",
        recurrence.project.as_deref().unwrap_or("N/A")
    );
    println!(
        "Priority:\t{}",
        recurrence.priority.as_deref().unwrap_or("N/A")
    );
    println!("Due:\t\t{}", recurrence.due.format(DATE_FORMAT));
    println!("Days between:\t{}", recurrence.days_between_recurrences);
}

fn create_recurrence(opts: CreateRecurrenceOpts) -> Result<()> {
    let recurrence = make_request(
        Method::Post,
        make_url("recurrence"),
        Some(NewRecurrenceTemplate::from(opts)),
    )
    .context("Unable to create recurrence")?
    .json::<RecurrenceTemplate>()?;

    println!("Successfully created recurrence.");
    print_recurrence(&recurrence);
    Ok(())
}

pub fn run() -> Result<()> {
    dotenv::dotenv().ok();

    let opts = Opts::parse();

    match opts.subcommand {
        SubCommand::Complete(CompleteOpts { task_id }) => complete_task(task_id),
        SubCommand::Create(create_opts) => create_task(create_opts),
        SubCommand::Info(InfoOpts { task_id }) => task_info(task_id),
        SubCommand::List(ListOpts { all }) => list_tasks(all),
        SubCommand::Modify(modify_opts) => modify_task(modify_opts),
        SubCommand::Uncomplete(UncompleteOpts { task_id }) => uncomplete_task(task_id),
        SubCommand::Recur(recur) => match recur.subcommand {
            RecurSubCommand::Create(create_opts) => create_recurrence(create_opts),
        },
    }
}
