// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        email -> Varchar,
        google_id -> Nullable<Varchar>,
        pass_hash -> Varchar,
        birthday -> Date,
        creation_time -> Timestamptz,
    }
}
