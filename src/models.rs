use super::schema::contacts;
use super::schema::transactions;
use super::schema::users_t;
use crate::inputting_status::InputtingStatus;
use bigdecimal::BigDecimal;
use diesel::prelude::*;
use diesel::{Identifiable, Queryable};

#[derive(Debug, Queryable, Selectable, Identifiable)]
#[diesel(table_name = users_t)]
pub struct User {
    pub id: i32,
    pub telegram_id: i64,
    pub telegram_username: String,
    pub status: InputtingStatus,
    pub selected_contact_id: Option<i32>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = users_t)]
pub struct NewUser {
    pub telegram_id: i64,
    pub telegram_username: String,
}

#[derive(Debug, Queryable, Identifiable, Associations)]
#[diesel(table_name = contacts)]
#[diesel(belongs_to(User, foreign_key = user_id))]
pub struct Contact {
    pub id: i32,
    pub user_id: i32,
    pub contact_id: i32,
    pub name: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = contacts)]
pub struct NewContact {
    pub user_id: i32,
    pub contact_id: i32,
}

#[derive(Debug, Queryable, Selectable, Identifiable, Associations)]
#[diesel(table_name = transactions)]
#[diesel(belongs_to(User, foreign_key = from_user_id))]
pub struct Transaction {
    pub id: i32,
    pub from_user_id: i32,
    pub to_user_id: i32,
    pub amount: BigDecimal,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = transactions)]
pub struct NewTransaction {
    pub from_user_id: i32,
    pub to_user_id: i32,
    pub amount: BigDecimal,
}
