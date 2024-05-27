// @generated automatically by Diesel CLI.

diesel::table! {
    sketches (id) {
        id -> Int4,
        creator_id -> Int4,
        creation_time -> Timestamptz,
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

diesel::joinable!(sketches -> users (creator_id));

diesel::allow_tables_to_appear_in_same_query!(
    sketches,
    users,
);
