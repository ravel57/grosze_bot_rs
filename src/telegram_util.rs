use crate::db_util;
use crate::inputting_status::InputtingStatus;
use crate::models::Transaction;
use crate::models::User;
use crate::HandlerResult;
use bigdecimal::BigDecimal;
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
use teloxide::types::MessageId;
use teloxide::utils::command::BotCommands;
use teloxide::utils::command::ParseError;
use teloxide::Bot;

const CALLBACK_SELECT_USER_PREFIX: &'static str = "selected_contact_";

/*
  TODO
    Комментарии к долгу
    Подтверждения
    Расчитались
    Кнопка НАЗАД В МЕНЮ
*/

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
    SelectContact,
    Debts,
    AddNewContact,
    EditContact,
    DeleteContact,
    TransactionDirectionGave,
    TransactionDirectionTook,
    TransactionSettledAccounts,
    TransactionHistory,
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
    let msg_text = msg.text().expect("ERROR getting message text").to_string();
    match user.status {
        InputtingStatus::None => {
            bot.send_message(telegram_id, "Никакого действия не выбрано, зайди в /menu")
                .await?;
        }
        InputtingStatus::NewContactTelegramUsername => {
            let username = msg_text.replace("@", "");
            let result = add_new_contact(&user, &username);
            match result {
                Ok(contact) => {
                    bot.send_message(telegram_id, "Пришли как ты хочешь подписать этот контакт")
                        .await
                        .expect("ERROR executing NewContactTelegramUsername");
                    db_util::set_selected_contact(&user, contact.id)
                        .expect("ERROR executing NewContactTelegramUsername");
                    set_user_status(&user, &InputtingStatus::NewContactInternalName);
                }
                Err(_) => {
                    bot.send_message(telegram_id, "Пользователь не найден\nСкорее всего он не зарегестирован в боте или ошибка в имени\nПришли еще раз или перейди в /menu")
						.await
						.expect("ERROR executing NewContactTelegramUsername");
                }
            }
        }
        InputtingStatus::NewContactInternalName => {
            let contact = db_util::get_selected_contact(&user).unwrap();
            let result = edit_contact(&user, &contact, &msg_text);
            match result {
                Ok(_) => {
                    bot.send_message(telegram_id, "Готово")
                        .await
                        .expect("ERROR executing NewContactInternalName");
                }
                Err(_) => {
                    bot.send_message(telegram_id, "Ошибка")
                        .await
                        .expect("ERROR executing NewContactInternalName");
                }
            };
            send_menu(&bot, telegram_id).await;
        }
        InputtingStatus::EditContactInternalName => {
            let contact = db_util::get_selected_contact(&user).unwrap();
            edit_contact(&user, &contact, &msg_text).expect("ERROR executing EditContact callback");
            bot.send_message(telegram_id, "Готово")
                .await
                .expect("ERROR executing EditContactInternalName");
            send_menu(&bot, telegram_id).await;
        }
        InputtingStatus::DeleteContact => { /*TODO*/ }
        InputtingStatus::SelectContactForTransaction => {}
        InputtingStatus::TransactionAmount => {
            let contact = db_util::get_selected_contact(&user).unwrap();
            if user.selected_transaction_duration.eq(&Option::from(0)) {
                create_transaction(
                    &user,
                    &contact,
                    msg_text.as_str(),
                    &bot,
                    telegram_id.clone(),
                )
                .await;
            } else {
                create_transaction(
                    &contact,
                    &user,
                    msg_text.as_str(),
                    &bot,
                    telegram_id.clone(),
                )
                .await;
            }
        }
    }
    Ok(())
}

async fn handle_command(bot: Bot, msg: Message) -> HandlerResult {
    let telegram_id = msg.chat.id;
    if let Some(text) = msg.text() {
        match Command::parse(text, "") {
            Ok(Command::Start) => {
                let username = msg
                    .chat
                    .username()
                    .expect("ERROR Username is not null")
                    .to_string();
                db_util::find_or_create_user(telegram_id.0, &username);
                send_menu(&bot, telegram_id.clone()).await;
            }
            Ok(Command::Menu) => {
                send_menu(&bot, telegram_id.clone()).await;
            }
            Ok(Command::Debts) => {
                let username = msg
                    .chat
                    .username()
                    .expect("ERROR Username is not null")
                    .to_string();
                let user = db_util::find_or_create_user(telegram_id.0, &username);
                let summary: Vec<(String, BigDecimal)> = get_debit(&user).expect("DB error");
                let text = summary
                    .iter()
                    .map(|(name, amount)| format!("{}: {}", name, amount))
                    .collect::<Vec<_>>()
                    .join("\n");
                bot.send_message(telegram_id, text)
                    .await
                    .expect("ERROR executing getting debits");
                send_menu(&bot, telegram_id.clone()).await;
            }
            Ok(Command::Contacts) => {
                let user = db_util::get_user_by_telegram_id(telegram_id.0).unwrap();
                let contacts_str = get_contacts_names(&user).join("\n");
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
    let mut user = db_util::get_user_by_telegram_id(telegram_id.0 as i64).unwrap(); //FIXME если нет пользователя то приложение падает
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
                send_contacts(&bot, &user, telegram_id.clone(), message_id.clone())
                    .await
                    .expect("ERROR executing SelectContact");
                set_user_status(&user, &InputtingStatus::SelectContactForTransaction);
            }
            Ok(MenuCommand::EditContact) => {
                send_contacts(&bot, &user, telegram_id.clone(), message_id.clone())
                    .await
                    .expect("ERROR executing EditContact");
                set_user_status(&user, &InputtingStatus::EditContactInternalName);
            }
            Ok(MenuCommand::DeleteContact) => {
                // TODO
                bot.send_message(telegram_id, "MenuCommand::DeleteContact")
                    .await
                    .expect("ERROR executing DeleteContact");
            }
            Ok(MenuCommand::TransactionDirectionGave) => {
                user.selected_transaction_duration = Option::from(0);
                db_util::set_selected_transaction_duration(&user, 0)
                    .expect("ERROR executing TransactionDirectionGave");
                set_user_status(&user, &InputtingStatus::TransactionAmount);
                let text = "Пришли сумму или вернись в /menu для отмены";
                bot.edit_message_text(telegram_id, message_id, text)
                    .await
                    .expect("ERROR executing EditContact");
            }
            Ok(MenuCommand::TransactionDirectionTook) => {
                user.selected_transaction_duration = Option::from(1);
                db_util::set_selected_transaction_duration(&user, 1)
                    .expect("ERROR executing TransactionDirectionTook");
                set_user_status(&user, &InputtingStatus::TransactionAmount);
                let text = "Пришли сумму или вернись в /menu для отмены";
                bot.edit_message_text(telegram_id, message_id, text)
                    .await
                    .expect("ERROR executing EditContact");
            }
            Ok(MenuCommand::Debts) => {
                let summary: Vec<(String, BigDecimal)> = get_debit(&user).expect("DB error");
                let text = summary
                    .iter()
                    .map(|(name, amount)| format!("{}: {}", name, amount))
                    .collect::<Vec<_>>()
                    .join("\n");
                bot.edit_message_text(telegram_id, message_id, text)
                    .await
                    .expect("ERROR executing getting debits");
            }
            Ok(MenuCommand::TransactionSettledAccounts) => { /*TODO*/ }
            Ok(MenuCommand::TransactionHistory) => { /*TODO*/ }
            Err(_) => {
                if data.starts_with(CALLBACK_SELECT_USER_PREFIX) {
                    handle_callback_for_selected_user(&data, &user, &bot, telegram_id, message_id)
                        .await;
                } else {
                    let text = format!("Необработанное нажатие:\n\"{data}\"");
                    bot.edit_message_text(telegram_id, message_id, text).await?;
                }
            }
        }
    }
    Ok(())
}

async fn handle_callback_for_selected_user(
    data: &String,
    user: &User,
    bot: &Bot,
    telegram_id: UserId,
    message_id: MessageId,
) {
    let contact_name = data.replace(CALLBACK_SELECT_USER_PREFIX, "");
    let contact =
        db_util::find_user_by_contact_name(user, &contact_name).expect("ERROR executing Username");
    db_util::set_selected_contact(user, contact.id).expect("ERROR executing Username");
    match user.status {
        InputtingStatus::EditContactInternalName => {
            let text = "Пришли новое имя или вернись в /menu для отмены";
            bot.edit_message_text(telegram_id, message_id, text)
                .await
                .expect("ERROR executing EditContact");
        }
        InputtingStatus::SelectContactForTransaction => {
            let keyboard = InlineKeyboardMarkup::new(vec![
                vec![
                    InlineKeyboardButton::callback(
                        "Взял в долг",
                        MenuCommand::TransactionDirectionGave.to_string(),
                    ),
                    InlineKeyboardButton::callback(
                        "Дал в долг",
                        MenuCommand::TransactionDirectionTook.to_string(),
                    ),
                ],
                vec![
                    InlineKeyboardButton::callback(
                        "История",
                        MenuCommand::TransactionHistory.to_string(),
                    ),
                    InlineKeyboardButton::callback(
                        "Расчитались",
                        MenuCommand::TransactionSettledAccounts.to_string(),
                    ),
                ],
            ]);
            bot.edit_message_text(telegram_id, message_id, "Выбери:")
                .await
                .expect("ERROR executing SelectContactForTransaction");
            bot.edit_message_reply_markup(telegram_id, message_id)
                .reply_markup(keyboard)
                .await
                .expect("ERROR executing SelectContactForTransaction");
        }
        // InputtingStatus::SelectDirectionForTransaction => {
        //     set_user_status(user, &InputtingStatus::TransactionAmount);
        //     let text = "Пришли сумму или вернись в /menu для отмены";
        //     bot.edit_message_text(telegram_id, message_id, text)
        //         .await
        //         .expect("ERROR executing EditContact");
        // }
        _ => {
            let text = format!("Необработанное нажатие:\n\"{data}\"");
            bot.edit_message_text(telegram_id, message_id, text)
                .await
                .expect("ERROR executing handle_callback_for_selected_user");
        }
    }
}

fn set_user_status(user: &User, new_status: &InputtingStatus) {
    db_util::set_user_status(user, new_status).expect("ERROR setting user status");
}

fn add_new_contact(user: &User, new_contact_name: &String) -> Result<User, Error> {
    match db_util::get_user_by_username(new_contact_name) {
        Ok(contact) => {
            db_util::find_or_create_contact(user, &contact);
            set_user_status(user, &InputtingStatus::NewContactInternalName);
            Ok(contact)
        }
        Err(e) => Err(e),
    }
}

fn edit_contact(user: &User, contact: &User, contact_new_name: &String) -> Result<(), Error> {
    match db_util::edit_contact(user, contact, contact_new_name) {
        Ok(_) => {
            set_user_status(user, &InputtingStatus::None);
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn get_contacts_names(user: &User) -> Vec<String> {
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

fn get_debit(user: &User) -> Option<Vec<(String, BigDecimal)>> {
    match db_util::get_debit(user) {
        Ok(el) => Some(el),
        Err(_) => None,
    }
}

async fn send_menu(bot: &Bot, telegram_id: ChatId) {
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("Долги", MenuCommand::SelectContact.to_string()),
            InlineKeyboardButton::callback("Сводка", MenuCommand::Debts.to_string()),
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
    let user = db_util::get_user_by_telegram_id(telegram_id.0).unwrap();
    set_user_status(&user, &InputtingStatus::None);
    bot.send_message(telegram_id, "Выбери действие:")
        .reply_markup(keyboard)
        .await
        .expect("ERROR creating menu");
}

async fn send_contacts(
    bot: &Bot,
    user: &User,
    telegram_id: UserId,
    message_id: MessageId,
) -> Result<(), Error> {
    let mut current_line: Vec<InlineKeyboardButton> = vec![];
    let mut lines: Vec<Vec<InlineKeyboardButton>> = vec![];
    let mut buttons_in_line: i8 = 0;
    for contact_name in get_contacts_names(user) {
        let callback_data = format!("{CALLBACK_SELECT_USER_PREFIX}{}", &contact_name);
        current_line.push(InlineKeyboardButton::callback(&contact_name, callback_data));
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
    if !&lines.is_empty() {
        bot.edit_message_text(telegram_id, message_id, "Выбери контакт:")
            .await
            .expect("ERROR executing send_contacts");
        bot.edit_message_reply_markup(telegram_id, message_id)
            .reply_markup(InlineKeyboardMarkup::new(lines))
            .await
            .expect("ERROR executing send_contacts");
    }
    Ok(())
}

async fn create_transaction(
    from: &User,
    to: &User,
    msg_text: &str,
    bot: &Bot,
    telegram_id: ChatId,
) -> Option<Transaction> {
    match db_util::create_transaction(from, to, msg_text) {
        Ok(transaction) => {
            bot.send_message(telegram_id, "Готово")
                .await
                .expect("ERROR execute TransactionAmount");
            send_menu(bot, telegram_id).await;
            Some(transaction)
        }
        Err(_) => {
            bot.send_message(telegram_id, "Ошибка парсинга суммы")
                .await
                .expect("ERROR execute TransactionAmount");
            None
        }
    }
}
