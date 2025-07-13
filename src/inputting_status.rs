use diesel::deserialize::FromSql;
use diesel::serialize::ToSql;
use diesel_derive_enum::DbEnum;
use strum_macros::EnumString;
use strum_macros::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, DbEnum, EnumString, Display)]
#[ExistingTypePath = "crate::schema::sql_types::Inputting_status"]
#[strum(serialize_all = "snake_case")]
pub enum InputtingStatus {
	#[db_rename = "none"]
	None,
	#[db_rename = "new_contact_telegram_username"]
	NewContactTelegramUsername,
	#[db_rename = "new_contact_internal_name"]
	NewContactInternalName,
	#[db_rename = "transaction_amount"]
	TransactionAmount,
}