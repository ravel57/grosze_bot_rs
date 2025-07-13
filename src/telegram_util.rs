use crate::db_util;
use crate::inputting_status::InputtingStatus;
use crate::models::User;
use crate::HandlerResult;
use diesel::prelude::*;
use diesel::result::Error;
use std::str::FromStr;
use strum_macros::Display;
use strum_macros::EnumString;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::*;
use teloxide::types::InlineKeyboardButton;
use teloxide::types::InlineKeyboardMarkup;
use teloxide::utils::command::BotCommands;
use teloxide::utils::command::ParseError;
use teloxide::{errors, Bot};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    #[command()]
    Start,
    #[command()]
    Debts,
    #[command()]
    Contacts,
    #[command()]
    Menu,
}

#[derive(EnumString, Display, Debug)]
#[strum(serialize_all = "snake_case")]
enum MenuCommand {
    AddNewContact,
    SelectContact,
    EditContact,
    DeleteContact,
    Debts,
}

pub fn message_handler_schema() -> Handler<'static, HandlerResult, DpHandlerDescription> {
    let commands = Update::filter_message()
        .filter(|msg: Message| msg.text().map(|t| t.starts_with('/')).unwrap_or(false))
        .endpoint(handle_command);
    let messages = Update::filter_message()
        .filter(|msg: Message| !msg.text().map(|t| t.starts_with('/')).unwrap_or(false))
        .endpoint(handle_message);
    let callbacks = Update::filter_callback_query().endpoint(handle_callback);
    dptree::entry()
        .branch(commands)
        .branch(messages)
        .branch(callbacks)
}

async fn handle_message(bot: Bot, msg: Message) -> HandlerResult {
    let telegram_id = msg.chat.id;
    let user = db_util::get_user_by_telegram_id(telegram_id.0).unwrap();
    match user.status {
        InputtingStatus::None => { /*TODO*/ }
        InputtingStatus::NewContactTelegramUsername => {
            let result = add_new_contact(
                &user,
                &msg.text().expect("ERROR getting message text").to_string(),
            );
            match result {
                Ok(_) => {
                    bot.send_message(telegram_id, "Пришли как ты хочешь подписать этот контакт")
                        .await
                        .expect_err("ERROR executing NewContactTelegramUsername");
                    set_user_status(&user, &InputtingStatus::TransactionAmount)
                }
                Err(_) => {
                    bot.send_message(telegram_id, "Пользователь не найден")
                        .await
                        .expect_err("ERROR executing NewContactTelegramUsername");
                }
            }
        }
        InputtingStatus::TransactionAmount => { /*TODO*/ }
    }
    Ok(())
}

async fn handle_command(bot: Bot, msg: Message) -> HandlerResult {
    let telegram_id = msg.chat.id;
    if let Some(text) = msg.text() {
        match Command::parse(text, "") {
            Ok(Command::Start) => {
                db_util::find_or_create_user(
                    telegram_id.0,
                    &msg.chat
                        .username()
                        .expect("ERROR Username is not null")
                        .to_string(),
                );
            }
            Ok(Command::Menu) => {
                let keyboard = InlineKeyboardMarkup::new(vec![
                    vec![
                        InlineKeyboardButton::callback(
                            "Выбрать контакт",
                            MenuCommand::SelectContact.to_string(),
                        ),
                        InlineKeyboardButton::callback(
                            "Общий расчет",
                            MenuCommand::Debts.to_string(),
                        ),
                    ],
                    vec![
                        InlineKeyboardButton::callback(
                            "Добавить контакт",
                            MenuCommand::AddNewContact.to_string(),
                        ),
                        InlineKeyboardButton::callback(
                            "Редактировать контакт",
                            MenuCommand::EditContact.to_string(),
                        ),
                        InlineKeyboardButton::callback(
                            "Удалить контакт",
                            MenuCommand::DeleteContact.to_string(),
                        ),
                    ],
                ]);
                bot.send_message(telegram_id, "Выберите действие:")
                    .reply_markup(keyboard)
                    .await
                    .expect("Не удалось отправить меню");
            }
            Ok(Command::Debts) => {
                bot.send_message(telegram_id, get_debit())
                    .await
                    .expect("ERROR executing getting debits");
            }
            Ok(Command::Contacts) => {
                let user = db_util::get_user_by_telegram_id(telegram_id.0).unwrap();
                let contacts_str = get_contact_names(&user).join("\n");
                bot.send_message(telegram_id, format!("Твои контакты:\n{contacts_str}"))
                    .await
                    .expect("ERROR executing getting contacts");
            }
            Err(_) => {}
        }
    }
    Ok(())
}

async fn handle_callback(bot: Bot, callback: CallbackQuery) -> HandlerResult {
    let telegram_id = callback.from.id;
    let message_id = callback.message.expect("Message ID not found").id();
    let user = db_util::get_user_by_telegram_id(telegram_id.0 as i64).unwrap(); //FIXME если нет пользователя то приложение падает
    bot.answer_callback_query(callback.id.clone()).await?;
    if let Some(data) = callback.data {
        match data.parse::<MenuCommand>() {
            Ok(MenuCommand::AddNewContact) => {
                bot.edit_message_text(
                    telegram_id,
                    message_id,
                    "Пришли telegram username нового контакта",
                )
                .await?;
                set_user_status(&user, &InputtingStatus::NewContactTelegramUsername);
            }
            Ok(MenuCommand::SelectContact) => {
                let mut current_line: Vec<InlineKeyboardButton> = vec![];
                let mut lines: Vec<Vec<InlineKeyboardButton>> = vec![];
                let mut buttons_in_line: i8 = 0;
                for contact_name in get_contact_names(&user) {
                    current_line.push(InlineKeyboardButton::callback(
                        &contact_name,
                        format!("selected_contact_{}", &contact_name),
                    ));
                    buttons_in_line += 1;
                    if buttons_in_line >= 3 {
                        lines.push(current_line.clone());
                        current_line.clear();
                        buttons_in_line = 0;
                    }
                }
                if !current_line.is_empty() {
                    lines.push(current_line.clone());
                }
                if !lines.is_empty() {
                    bot.edit_message_text(telegram_id, message_id, "Выбери контакт:")
                        .await
                        .expect("ERROR executing SelectContact callback");
                    bot.edit_message_reply_markup(telegram_id, message_id)
                        .reply_markup(InlineKeyboardMarkup::new(lines))
                        .await
                        .expect("ERROR executing SelectContact callback");
                }
            }
            Ok(MenuCommand::EditContact) => {
                panic!("TODO")
            }
            Ok(MenuCommand::DeleteContact) => {
                panic!("TODO")
            }
            Ok(MenuCommand::Debts) => {
                bot.edit_message_text(telegram_id, message_id, get_debit())
                    .await
                    .expect("ERROR executing getting debits");
            }
            Err(_) => {
                bot.edit_message_text(
                    telegram_id,
                    message_id,
                    format!("Необработанное нажатие: {data}"),
                )
                .await?;
            }
        }
    }
    Ok(())
}

fn add_new_contact(user: &User, new_contact_name: &String) -> Result<(), diesel::result::Error> {
    match db_util::get_user_by_username(new_contact_name) {
        Ok(contact) => {
            db_util::find_or_create_contact(user, &contact);
            set_user_status(user, &InputtingStatus::None);
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn get_debit() -> String {
    // let mut s = String::new();
    // for (tx, from_user, to_user) in db.get_debts().await? {
    //     s.push_str(&format!(
    //         "{} → {}: {} ₽\n",
    //         from_user.name, to_user.name, tx.amount
    //     ));
    // }
    "String::new()".to_string()
}

fn set_user_status(user: &User, new_status: &InputtingStatus) {
    db_util::set_user_status(user, new_status).expect("ERROR setting user status");
}

fn get_contact_names(user: &User) -> Vec<String> {
    db_util::find_all_contacts_for_user(user)
        .iter()
        .map(|contact| {
            contact.name.clone().unwrap_or_else(|| {
                db_util::find_or_create_user(contact.id as i64, &String::new())
                    .telegram_username
                    .clone()
            })
        })
        .collect::<Vec<_>>()
}
