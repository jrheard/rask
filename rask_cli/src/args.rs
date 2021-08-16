use chrono::{format::ParseResult, NaiveDate};
use clap::Clap;
use rask_lib::models;

#[derive(Debug)]
pub enum ParseDecision<T> {
    Set(T),
    Delete,
}

// Clap seems to treat an Ok(None) value as "this arg was unspecified", so we use
// an eerily-Option-like ParseDecision enum to represent the situation where the user gave us
// the string "none".
fn parse_date_str_or_none_str(date_str: &str) -> ParseResult<ParseDecision<NaiveDate>> {
    if date_str == "none" {
        Ok(ParseDecision::Delete)
    } else {
        Ok(ParseDecision::Set(NaiveDate::parse_from_str(
            date_str,
            crate::DATE_FORMAT,
        )?))
    }
}

fn parse_date(date_str: &str) -> ParseResult<NaiveDate> {
    NaiveDate::parse_from_str(date_str, crate::DATE_FORMAT)
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
    Uncomplete(UncompleteOpts),
    Recur(Recur),
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
pub struct ListOpts {
    #[clap(long)]
    pub all: bool,
}
#[derive(Clap, Debug)]
pub struct CreateOpts {
    pub name: String,

    #[clap(long, alias = "proj", parse(try_from_str = parse_project))]
    pub project: Option<String>,

    #[clap(long, alias = "prio", possible_values(&["H", "M", "L"]))]
    pub priority: Option<String>,

    /// Format: MM/DD/YYYY, e.g. 05/01/2021
    #[clap(short, long, parse(try_from_str = parse_date))]
    pub due: Option<NaiveDate>,
}

impl From<CreateOpts> for models::NewTask {
    fn from(
        CreateOpts {
            name,
            project,
            priority,
            due,
        }: CreateOpts,
    ) -> Self {
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
    pub due: Option<ParseDecision<NaiveDate>>,
}

#[derive(Clap)]
pub struct UncompleteOpts {
    pub task_id: i32,
}

#[derive(Clap)]
pub struct Recur {
    #[clap(subcommand)]
    pub subcommand: RecurSubCommand,
}

#[derive(Clap)]
pub enum RecurSubCommand {
    Create(CreateRecurrenceOpts),
}
#[derive(Clap)]
pub struct CreateRecurrenceOpts {
    pub name: String,

    /// Format: MM/DD/YYYY, e.g. 05/01/2021
    #[clap(short, long, parse(try_from_str = parse_date))]
    pub due: NaiveDate,

    pub days_between_recurrences: i32,

    #[clap(long, alias = "proj", parse(try_from_str = parse_project))]
    pub project: Option<String>,

    #[clap(long, alias = "prio", possible_values(&["H", "M", "L"]))]
    pub priority: Option<String>,
}

impl From<CreateRecurrenceOpts> for models::NewRecurrenceTemplate {
    fn from(
        CreateRecurrenceOpts {
            name,
            project,
            priority,
            due,
            days_between_recurrences,
        }: CreateRecurrenceOpts,
    ) -> Self {
        models::NewRecurrenceTemplate {
            name,
            project,
            priority,
            due,
            days_between_recurrences,
        }
    }
}
