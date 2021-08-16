table! {
    api_token (token) {
        token -> Text,
    }
}

table! {
    recurrence_template (id) {
        id -> Int4,
        time_created -> Timestamptz,
        name -> Text,
        project -> Nullable<Text>,
        priority -> Nullable<Text>,
        due -> Date,
        days_between_recurrences -> Int4,
    }
}

table! {
    task (id) {
        id -> Int4,
        name -> Text,
        project -> Nullable<Text>,
        priority -> Nullable<Text>,
        mode -> Text,
        time_created -> Timestamptz,
        due -> Nullable<Date>,
        recurrence_template_id -> Nullable<Int4>,
    }
}

joinable!(task -> recurrence_template (recurrence_template_id));

allow_tables_to_appear_in_same_query!(
    api_token,
    recurrence_template,
    task,
);
