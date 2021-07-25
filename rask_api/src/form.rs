use crate::models::{NewTask, PRIORITY_HIGH, PRIORITY_LOW, PRIORITY_MEDIUM};
use chrono::NaiveDateTime;
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
pub struct NaiveDateTimeFormField(NaiveDateTime);

#[rocket::async_trait]
impl<'r> FromFormField<'r> for NaiveDateTimeFormField {
    fn from_value(form_value: ValueField<'r>) -> form::Result<'r, Self> {
        let parsed = NaiveDateTime::parse_from_str(form_value.value, "%Y-%m-%d %H:%M:%S");

        match parsed {
            Ok(naive_date_time) => Ok(NaiveDateTimeFormField(naive_date_time)),
            Err(_) => Err(form::Error::validation("invalid datetime").into()),
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
    due: Option<NaiveDateTimeFormField>,
}

impl From<Form<TaskForm>> for NewTask {
    fn from(form: Form<TaskForm>) -> Self {
        let form = form.into_inner();
        NewTask {
            name: form.name,
            project: form.project,
            priority: form.priority,
            due: form.due.map(|due| due.0),
        }
    }
}
