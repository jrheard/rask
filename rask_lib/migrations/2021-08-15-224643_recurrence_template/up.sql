CREATE TABLE recurrence_template (
    id SERIAL PRIMARY KEY,
    time_created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    name TEXT NOT NULL,
    project TEXT,
    priority TEXT,
    due DATE NOT NULL,
    days_between_recurrences integer NOT NULL
);

ALTER TABLE task
    ADD COLUMN recurrence_template_id INTEGER,
    ADD CONSTRAINT fk_task_recurrence_template
    FOREIGN KEY (recurrence_template_id)
    REFERENCES recurrence_template (id);