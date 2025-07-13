use diesel::{allow_tables_to_appear_in_same_query, joinable, table};

table! {
    users_t (id) {
        id -> Int4,
        telegram_id -> Int8,
        telegram_username -> Text,
    }
}

table! {
    contacts (id) {
        id -> Int4,
        user_id -> Int4,
        contact_id -> Int4,
        name -> Text,
    }
}

table! {
    transactions (id) {
        id -> Int4,
        from_user_id -> Int4,
        to_user_id -> Int4,
        amount -> Numeric,
    }
}

joinable!(contacts -> users_t (user_id));
joinable!(transactions -> users_t (from_user_id));

allow_tables_to_appear_in_same_query!(
    users_t,
    contacts,
    transactions,
);