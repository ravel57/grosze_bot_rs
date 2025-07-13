use std::error::Error;
use bigdecimal::BigDecimal;
use diesel::{PgConnection, RunQueryDsl};
use diesel::upsert::excluded;
use crate::models::{Contact, NewContact, NewTransaction, NewUser, Transaction, User};
use crate::schema::transactions::dsl::transactions;

/// Вставляет нового пользователя или обновляет поле `telegram_username`, если telegram_id уже существует
pub fn find_or_create_user(conn: &mut PgConnection, tg_id: i64, tg_username: String) -> User {
    let new_user = NewUser {
        telegram_id: tg_id,
        telegram_username: tg_username.clone(),
    };
    diesel::insert_into(crate::schema::users_t::dsl::users_t)
        .values(&new_user)
        .on_conflict(crate::schema::users_t::dsl::telegram_id)
        .do_update()
        .set(crate::schema::users_t::dsl::telegram_username.eq(excluded(crate::schema::users_t::dsl::telegram_username)))
        .get_result(conn)
        .expect("Error creating user")
}

/// Вставляем контакт, игнорируем ошибку, если уже есть такая пара (user_id, contact_id)
pub fn find_or_create_contact(
    conn: &mut PgConnection,
    user: &User,
    contact: &User,
    name: &str,
) -> Contact {
    let new_contact = NewContact {
        user_id: user.id,
        contact_id: contact.id,
        name: name.to_owned(),
    };
    diesel::insert_into(crate::schema::contacts::dsl::contacts)
        .values(&new_contact)
        .on_conflict((crate::schema::contacts::dsl::user_id, crate::schema::contacts::dsl::contact_id))
        .do_nothing()
        .get_result(conn)
        .or_else(|err| {
            if let diesel::result::Error::NotFound = err {
                crate::schema::contacts::dsl::contacts
                    .filter(crate::schema::contacts::dsl::user_id.eq(user.id))
                    .filter(crate::schema::contacts::dsl::contact_id.eq(contact.id))
                    .first(conn)
            } else {
                Err(err)
            }
        })
        .expect("Error creating or fetching contact")
}

pub fn find_all_contacts_for_user(
    conn: &mut PgConnection,
    user_id_value: i32,
) -> Result<Vec<Contact>, Error> {
    contacts
        .filter(user_id.eq(user_id_value))
        .load::<Contact>(conn)
}

pub fn create_transaction(
    conn: &mut PgConnection,
    from: &User,
    to: &User,
    amt_str: &str,
) -> Transaction {
    let amt = BigDecimal::from_str(amt_str).unwrap();
    let new_tx = NewTransaction {
        from_user_id: from.id,
        to_user_id: to.id,
        amount: amt,
    };
    diesel::insert_into(transactions)
        .values(&new_tx)
        .get_result::<Transaction>(conn)
        .expect("Error creating transaction")
}