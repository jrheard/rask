use chrono::{format::ParseResult, NaiveDate, NaiveDateTime};
use clap::Clap;
use rask_lib::models;

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

fn parse_project(project: &str) -> Result<String, String> {
    if project.split(' ').count() == 1 {
        Ok(project.to_string())
    } else {
        Err("Project must be one word".to_string())
    }
}

fn parse_date(date_str: &str) -> ParseResult<NaiveDateTime> {
    Ok(NaiveDate::parse_from_str(date_str, crate::DATE_FORMAT)?.and_hms(0, 0, 0))
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
