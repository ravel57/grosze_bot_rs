use diesel::allow_tables_to_appear_in_same_query;
use diesel::joinable;
use diesel::sql_types::SqlType;
use diesel::table;

pub mod sql_types {
    use super::*;
    #[derive(SqlType)]
    #[diesel(postgres_type(name = "inputting_status"))]
    pub struct Inputting_status;
}

table! {
    users_t (id) {
        id -> Int4,
        telegram_id -> BigInt,
        telegram_username -> Text,
        status -> crate::schema::sql_types::Inputting_status,
        selected_contact_id -> Nullable<Integer>,
    }
}

table! {
    contacts (id) {
        id -> Int4,
        user_id -> Int4,
        contact_id -> Int4,
        name -> Nullable<Text>,
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

allow_tables_to_appear_in_same_query!(users_t, contacts, transactions,);
