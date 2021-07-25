use crate::models::{NewTask, PRIORITY_HIGH, PRIORITY_LOW, PRIORITY_MEDIUM};
use rocket::form;
use rocket::form::{Form, FromForm};

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

#[derive(FromForm)]
pub struct TaskForm {
    name: String,
    #[field(validate=validate_project())]
    project: Option<String>,
    #[field(validate=validate_priority())]
    priority: Option<String>,
}

impl From<Form<TaskForm>> for NewTask {
    fn from(form: Form<TaskForm>) -> Self {
        let form = form.into_inner();
        NewTask {
            name: form.name,
            project: form.project,
            priority: form.priority,
        }
    }
}
