use chrono::NaiveDate;
use rask_lib::models::{
    NewRecurrenceTemplate, NewTask, PRIORITY_HIGH, PRIORITY_LOW, PRIORITY_MEDIUM,
};
use rocket::form::{self, ValueField};
use rocket::form::{Form, FromForm, FromFormField};

/// Task projects must be a single word or None.
fn validate_project<'v>(project: &Option<String>) -> form::Result<'v, ()> {
    match project {
        Some(project) if project.split(' ').count() != 1 => {
            Err(form::Error::validation("project must be a single word or blank").into())
        }
        _ => Ok(()),
    }
}

/// Task priorities must be a valid Priority value or None.
fn validate_priority<'v>(priority: &Option<String>) -> form::Result<'v, ()> {
    match priority {
        None => Ok(()),
        Some(priority_str)
            if [PRIORITY_HIGH.0, PRIORITY_MEDIUM.0, PRIORITY_LOW.0]
                .iter()
                .any(|&v| v == priority_str) =>
        {
            Ok(())
        }
        _ => Err(form::Error::validation("priority must be one of H,M,L or blank").into()),
    }
}
pub struct NaiveDateFormField(NaiveDate);

#[rocket::async_trait]
impl<'r> FromFormField<'r> for NaiveDateFormField {
    fn from_value(form_value: ValueField<'r>) -> form::Result<'r, Self> {
        let parsed = NaiveDate::parse_from_str(form_value.value, "%Y-%m-%d");

        match parsed {
            Ok(naive_date_time) => Ok(NaiveDateFormField(naive_date_time)),
            Err(_) => Err(form::Error::validation("invalid date").into()),
        }
    }
}

#[derive(FromForm)]
pub struct TaskForm {
    name: String,
    #[field(validate=validate_project())]
    project: Option<String>,
    #[field(validate=validate_priority())]
    priority: Option<String>,
    due: Option<NaiveDateFormField>,
}

#[derive(FromForm)]
pub struct RecurrenceForm {
    name: String,
    #[field(validate=validate_project())]
    project: Option<String>,
    #[field(validate=validate_priority())]
    priority: Option<String>,
    due: NaiveDateFormField,
    days_between_recurrences: i32,
}

// Wrapper types to work around the orphan rule.
pub struct WrappedNewTask(pub NewTask);

impl From<Form<TaskForm>> for WrappedNewTask {
    fn from(form: Form<TaskForm>) -> Self {
        let form = form.into_inner();
        WrappedNewTask(NewTask {
            name: form.name,
            project: form.project,
            priority: form.priority,
            due: form.due.map(|due| due.0),
        })
    }
}

pub struct WrappedNewRecurrenceTemplate(pub NewRecurrenceTemplate);

impl From<Form<RecurrenceForm>> for WrappedNewRecurrenceTemplate {
    fn from(form: Form<RecurrenceForm>) -> Self {
        let form = form.into_inner();
        WrappedNewRecurrenceTemplate(NewRecurrenceTemplate {
            name: form.name,
            project: form.project,
            priority: form.priority,
            due: form.due.0,
            days_between_recurrences: form.days_between_recurrences,
        })
    }
}
