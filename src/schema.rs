// @generated automatically by Diesel CLI.

diesel::table! {
    friendships (friendship_id) {
        friendship_id -> Int4,
        user_one -> Int4,
        user_two -> Int4,
        status -> Int4,
    }
}

diesel::table! {
    users (user_id) {
        user_id -> Int4,
        name -> Varchar,
        username -> Varchar,
    }
}

diesel::table! {
    wishes (wish_id) {
        wish_id -> Int4,
        description -> Varchar,
        access_level -> Varchar,
        user_id -> Int4,
    }
}

diesel::joinable!(wishes -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    friendships,
    users,
    wishes,
);
