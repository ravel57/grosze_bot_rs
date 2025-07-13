use crate::establish_connection;
use crate::inputting_status::InputtingStatus;
use crate::models::Contact;
use crate::models::NewContact;
use crate::models::NewTransaction;
use crate::models::NewUser;
use crate::models::Transaction;
use crate::models::User;
use crate::schema::contacts::dsl as contacts_dsl;
use crate::schema::transactions::dsl as txs_dsl;
use crate::schema::users_t::dsl as users_dsl;
use bigdecimal::{BigDecimal, ParseBigDecimalError};
use diesel::prelude::*;
use diesel::prelude::*;
use diesel::prelude::*;
use diesel::result::QueryResult;
use diesel::upsert::excluded;
use diesel::PgConnection;
use diesel::RunQueryDsl;
use std::str::FromStr;

/// blablabla
pub fn get_user_by_telegram_id(tg_id_val: i64) -> QueryResult<User> {
    let mut conn = establish_connection();
    users_dsl::users_t
        .filter(users_dsl::telegram_id.eq(tg_id_val))
        .first(&mut conn)
}

/// blablabla
pub fn get_user_by_username(tg_username: &String) -> QueryResult<User> {
    let mut conn = establish_connection();
    users_dsl::users_t
        .filter(users_dsl::telegram_username.eq(tg_username))
        .first(&mut conn)
}

/// Вставляет или обновляет пользователя по telegram_id
pub fn find_or_create_user(tg_id_val: i64, tg_username_val: &String) -> User {
    let mut conn = establish_connection();
    let new_user = NewUser {
        telegram_id: tg_id_val,
        telegram_username: tg_username_val.clone(),
    };
    diesel::insert_into(users_dsl::users_t)
        .values(&new_user)
        .on_conflict(users_dsl::telegram_id)
        .do_update()
        .set(users_dsl::telegram_username.eq(excluded(users_dsl::telegram_username)))
        .get_result(&mut conn)
        .expect("Error creating or updating user")
}

/// Вставляет контакт; если уже есть пара (user_id, contact_id), берёт существующий
pub fn find_or_create_contact(user: &User, contact: &User) -> Contact {
    let mut conn = establish_connection();
    let new_contact = NewContact {
        user_id: user.id,
        contact_id: contact.id,
    };
    diesel::insert_into(contacts_dsl::contacts)
        .values(&new_contact)
        .on_conflict((contacts_dsl::user_id, contacts_dsl::contact_id))
        .do_nothing()
        .get_result(&mut conn)
        .or_else(|err| {
            if let diesel::result::Error::NotFound = err {
                contacts_dsl::contacts
                    .filter(contacts_dsl::user_id.eq(user.id))
                    .filter(contacts_dsl::contact_id.eq(contact.id))
                    .first(&mut conn)
            } else {
                log::error!("Error: {:?}", err);
                Err(err)
            }
        })
        .expect("Error creating or fetching contact")
}

/// Возвращает все контакты для данного user_id
pub fn find_all_contacts_for_user(user: &User) -> Vec<Contact> {
    let mut conn = establish_connection();
    contacts_dsl::contacts
        .filter(contacts_dsl::user_id.eq(user.id))
        .load::<Contact>(&mut conn)
        .expect("Error loading contacts")
}

/// Создаёт транзакцию между двумя пользователями
pub fn create_transaction(
    from: &User,
    to: &User,
    amount_str: &str,
) -> Result<Transaction, ParseBigDecimalError> {
    let mut conn = establish_connection();
    let amount = BigDecimal::from_str(amount_str);
    match amount {
        Ok(amount) => {
            let new_tx = NewTransaction {
                from_user_id: from.id,
                to_user_id: to.id,
                amount,
            };
            let transaction = diesel::insert_into(txs_dsl::transactions)
                .values(&new_tx)
                .get_result::<Transaction>(&mut conn)
                .expect("Error creating transaction");
            Ok(transaction)
        }
        Err(e) => Err(e),
    }
}

pub fn set_user_status(user: &User, new_status: &InputtingStatus) -> QueryResult<User> {
    let mut conn = establish_connection();
    diesel::update(users_dsl::users_t.filter(users_dsl::id.eq(user.id)))
        .set(users_dsl::status.eq(new_status))
        .get_result(&mut conn)
}

pub fn edit_contact(
    user: &User,
    contact: &User,
    contact_new_name: &String,
) -> QueryResult<Contact> {
    let mut conn = establish_connection();
    diesel::update(
        contacts_dsl::contacts
            .filter(contacts_dsl::user_id.eq(user.id))
            .filter(contacts_dsl::contact_id.eq(contact.id)),
    )
    .set(contacts_dsl::name.eq(contact_new_name))
    .get_result(&mut conn)
}

pub fn set_selected_contact(user: &User, contact_id: i32) -> QueryResult<User> {
    let mut conn = establish_connection();
    diesel::update(users_dsl::users_t.filter(users_dsl::id.eq(user.id)))
        .set(users_dsl::selected_contact_id.eq(contact_id))
        .get_result(&mut conn)
}

pub fn get_selected_contact(user: &User) -> QueryResult<User> {
    let mut conn = establish_connection();
    if let Some(contact_id) = user.selected_contact_id {
        users_dsl::users_t
            .filter(users_dsl::id.eq(contact_id))
            .first(&mut conn)
    } else {
        Err(diesel::result::Error::NotFound)
    }
}

pub fn find_user_by_contact_name(user: &User, contact_name: &str) -> QueryResult<User> {
    let mut conn = establish_connection();
    contacts_dsl::contacts
        .inner_join(users_dsl::users_t.on(contacts_dsl::contact_id.eq(users_dsl::id)))
        .filter(contacts_dsl::user_id.eq(user.id))
        .filter(contacts_dsl::name.eq(contact_name))
        .select(users_dsl::users_t::all_columns())
        .first::<User>(&mut conn)
}

pub fn set_selected_transaction_duration(user: &User, direction: i32) -> QueryResult<User> {
    let mut conn: PgConnection = establish_connection();
    diesel::update(users_dsl::users_t.filter(users_dsl::id.eq(user.id)))
        .set(users_dsl::selected_transaction_duration.eq(direction))
        .get_result(&mut conn)
}

/// Возвращает сводку сумм, которые пользователь `user` перевёл каждому контакту,
/// где ключ — имя контакта, а значение — сумма переводов (BigDecimal).
pub fn get_debit(user: &User) -> QueryResult<Vec<(String, BigDecimal)>> {
    let mut conn = establish_connection();
    let rows: Vec<(Option<String>, Option<BigDecimal>)> = contacts_dsl::contacts
        .filter(contacts_dsl::user_id.eq(user.id))
        .inner_join(
            txs_dsl::transactions.on(
                contacts_dsl::contact_id.eq(txs_dsl::to_user_id),
            )
        )
        .filter(txs_dsl::from_user_id.eq(user.id))
        .group_by(contacts_dsl::name)
        .select((
            contacts_dsl::name,
            diesel::dsl::sum(txs_dsl::amount),
        ))
        .load(&mut conn)?;
    let summary = rows
        .into_iter()
        .map(|(name_opt, sum_opt)| {
            let name = name_opt.unwrap_or_else(|| "<unknown>".to_string());
            let total = sum_opt.unwrap_or_else(|| BigDecimal::from(0));
            (name, total)
        })
        .collect();
    Ok(summary)
}
