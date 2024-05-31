// @generated automatically by Diesel CLI.

diesel::table! {
    sketch_groups (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        creator_id -> Int4,
    }
}

diesel::table! {
    sketches (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        creator_id -> Int4,
        creation_time -> Timestamptz,
        sketch_group -> Nullable<Int4>,
        data -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        email -> Varchar,
        google_id -> Nullable<Varchar>,
        pass_hash -> Varchar,
        birthday -> Date,
        grade -> Int2,
        creation_time -> Timestamptz,
    }
}

diesel::joinable!(sketch_groups -> users (creator_id));
diesel::joinable!(sketches -> sketch_groups (sketch_group));
diesel::joinable!(sketches -> users (creator_id));

diesel::allow_tables_to_appear_in_same_query!(
    sketch_groups,
    sketches,
    users,
);
