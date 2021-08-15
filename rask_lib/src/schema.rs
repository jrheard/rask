table! {
    api_token (token) {
        token -> Text,
    }
}

table! {
    task (id) {
        id -> Int4,
        name -> Text,
        project -> Nullable<Text>,
        priority -> Nullable<Text>,
        mode -> Text,
        due -> Nullable<Timestamp>,
        time_created -> Timestamptz,
    }
}

allow_tables_to_appear_in_same_query!(
    api_token,
    task,
);
