table! {
    task (id) {
        id -> Int4,
        name -> Text,
        project -> Nullable<Text>,
        priority -> Nullable<Text>,
        mode -> Text,
        due -> Nullable<Timestamp>,
    }
}
