table! {
    task (id) {
        id -> Int4,
        name -> Text,
        mode -> Text,
        project -> Nullable<Text>,
        priority -> Nullable<Text>,
    }
}
