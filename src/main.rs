use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::env;
use std::str::FromStr;
use teloxide::macros::BotCommands;
use teloxide::prelude::{Message, Requester, ResponseResult};
use teloxide::repls::CommandReplExt;
use teloxide::Bot;

mod db_commands;
mod models;
mod schema;

/// Встраиваем все миграции из каталога `migrations/`
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

#[tokio::main]
async fn main() {
    // let conn = establish_connection();
    pretty_env_logger::init();
    let bot = Bot::from_env();
    Command::repl(bot, answer).await;
}

fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let mut conn = PgConnection::establish(&database_url).expect("Error connecting to database");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");
    conn
}

/// These commands are supported:
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    #[command()]
    Debts,
    #[command()]
    Contacts,
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    let conn = establish_connection();
    match cmd {
        Command::Debts => {
            // let mut s = String::new();
            // for (tx, from_user, to_user) in db.get_debts().await? {
            //     s.push_str(&format!(
            //         "{} → {}: {} ₽\n",
            //         from_user.name, to_user.name, tx.amount
            //     ));
            // }
            bot.send_message(msg.chat.id, "s").await?;
        }
        Command::Contacts => {
            db_commands::find_or_create_contact()
            bot.send_message(msg.chat.id, "Твои контакты:".to_string())
                .await?;
        }
    };
    Ok(())
}

/*
fn run_example(conn: &mut PgConnection) {
    let alice = find_or_create_user(conn, 123456789, "alice_bot".into());
    let bob = find_or_create_user(conn, 987654321, "bob_bot".into());
    find_or_create_contact(conn, &alice, &bob, "Bob Friend");
    create_transaction(conn, &alice, &bob, "42.50");
}
*/
