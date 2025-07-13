use diesel::{Connection, PgConnection};
use diesel_migrations::EmbeddedMigrations;
use diesel_migrations::{embed_migrations, MigrationHarness};
use std::env;
use teloxide::dispatching::Dispatcher;
use teloxide::Bot;

mod db_util;
mod inputting_status;
mod models;
mod schema;
mod telegram_util;

/// Встраиваем все миграции из каталога `migrations/`
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> HandlerResult {
    pretty_env_logger::init();

    let mut conn = establish_connection();
    conn.run_pending_migrations(MIGRATIONS).expect("Error applying migrations");

    let bot = Bot::from_env();
    Dispatcher::builder(bot, telegram_util::message_handler_schema())
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    Ok(())
}

pub fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let mut conn = PgConnection::establish(&database_url).expect("Error connecting to database");
    conn
}

/*
fn run_example(conn: &mut PgConnection) {
    let alice = find_or_create_user(conn, 123456789, "alice_bot".into());
    let bob = find_or_create_user(conn, 987654321, "bob_bot".into());
    find_or_create_contact(conn, &alice, &bob, "Bob Friend");
    create_transaction(conn, &alice, &bob, "42.50");
}
*/
