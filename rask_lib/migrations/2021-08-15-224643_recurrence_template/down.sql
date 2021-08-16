ALTER TABLE task DROP CONSTRAINT fk_task_recurrence_template;
ALTER TABLE task DROP COLUMN recurrence_template_id;

DROP TABLE recurrence_template;