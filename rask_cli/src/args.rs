use chrono::{format::ParseResult, NaiveDate, NaiveDateTime};
use clap::Clap;
use rask_lib::models;

#[derive(Debug)]
pub enum ParseDecision<T> {
    Set(T),
    Delete,
}

fn parse_date_str_or_none_str(date_str: &str) -> ParseResult<ParseDecision<NaiveDateTime>> {
    if date_str == "none" {
        Ok(ParseDecision::Delete)
    } else {
        Ok(ParseDecision::Set(
            NaiveDate::parse_from_str(date_str, crate::DATE_FORMAT)?.and_hms(0, 0, 0),
        ))
    }
}

fn parse_date(date_str: &str) -> ParseResult<NaiveDateTime> {
    Ok(NaiveDate::parse_from_str(date_str, crate::DATE_FORMAT)?.and_hms(0, 0, 0))
}

fn parse_project(project: &str) -> Result<String, String> {
    if project.split(' ').count() == 1 {
        Ok(project.to_string())
    } else {
        Err("Project must be one word".to_string())
    }
}

#[derive(Clap)]
pub struct Opts {
    #[clap(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Clap)]
pub enum SubCommand {
    Complete(CompleteOpts),
    Create(CreateOpts),
    Info(InfoOpts),
    List(ListOpts),
    Modify(ModifyOpts),
}
#[derive(Clap)]
pub struct CompleteOpts {
    pub task_id: i32,
}

#[derive(Clap)]
pub struct InfoOpts {
    pub task_id: i32,
}

#[derive(Clap)]
pub struct ListOpts {}
#[derive(Clap, Debug)]
pub struct CreateOpts {
    pub name: String,

    #[clap(long, alias = "proj", parse(try_from_str = parse_project))]
    pub project: Option<String>,

    #[clap(long, alias = "prio", possible_values(&["H", "M", "L"]))]
    pub priority: Option<String>,

    /// Format: MM/DD/YYYY, e.g. 05/01/2021
    #[clap(short, long, parse(try_from_str = parse_date))]
    pub due: Option<NaiveDateTime>,
}

impl From<CreateOpts> for models::NewTask {
    fn from(opts: CreateOpts) -> Self {
        let CreateOpts {
            name,
            project,
            priority,
            due,
        } = opts;

        models::NewTask {
            name,
            project,
            priority,
            due,
        }
    }
}

#[derive(Clap, Debug)]
pub struct ModifyOpts {
    pub task_id: i32,

    /// The task's new name, if you want to change the name.
    pub name: Option<String>,

    /// A one-word project name. A value of `none` deletes the project.
    #[clap(long, alias = "proj", parse(try_from_str = parse_project))]
    pub project: Option<String>,

    /// A value of `none` deletes the priority.
    #[clap(long, alias = "prio", possible_values(&["H", "M", "L", "none"]))]
    pub priority: Option<String>,

    /// Format: MM/DD/YYYY, e.g. 05/01/2021. A value of `none` deletes the due date.
    #[clap(short, long, parse(try_from_str = parse_date_str_or_none_str))]
    pub due: Option<ParseDecision<NaiveDateTime>>,
}
