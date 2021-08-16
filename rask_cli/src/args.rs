use chrono::{Datelike, NaiveDate, ParseError};
use clap::Clap;
use rask_lib::models;
use thiserror::Error;

#[derive(Debug)]
pub enum ParseDecision<T> {
    Set(T),
    Delete,
}

#[derive(Debug, Error)]
pub enum DateParseError {
    #[error("Error parsing date")]
    ChronoError(#[from] ParseError),

    #[error("Date's year was too low: {year:?} (specify MM/DD/YYYY, not MM/DD/YY)")]
    YearTooLowError { year: i32 },
}

// Clap seems to treat an Ok(None) value as "this arg was unspecified", so we use
// an eerily-Option-like ParseDecision enum to represent the situation where the user gave us
// the string "none".
fn parse_date_str_or_none_str(date_str: &str) -> Result<ParseDecision<NaiveDate>, DateParseError> {
    if date_str == "none" {
        Ok(ParseDecision::Delete)
    } else {
        parse_date(date_str).map(ParseDecision::Set)
    }
}

fn parse_date(date_str: &str) -> Result<NaiveDate, DateParseError> {
    NaiveDate::parse_from_str(date_str, crate::DATE_FORMAT)
        .map_err(DateParseError::ChronoError)
        .and_then(|date| {
            if date.year() >= 2000 {
                Ok(date)
            } else {
                Err(DateParseError::YearTooLowError { year: date.year() })
            }
        })
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
    Create(RecurrenceCreateOpts),
    Info(RecurrenceInfoOpts),
}

#[derive(Clap)]
pub struct RecurrenceInfoOpts {
    pub recurrence_id: i32,
}

#[derive(Clap)]
pub struct RecurrenceCreateOpts {
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

impl From<RecurrenceCreateOpts> for models::NewRecurrenceTemplate {
    fn from(
        RecurrenceCreateOpts {
            name,
            project,
            priority,
            due,
            days_between_recurrences,
        }: RecurrenceCreateOpts,
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
