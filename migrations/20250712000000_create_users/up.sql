CREATE TYPE inputting_status AS ENUM ('none', 'new_contact_telegram_username', 'new_contact_internal_name', 'transaction_amount');

CREATE TABLE IF NOT EXISTS users_t
(
    id                SERIAL PRIMARY KEY,
    telegram_id       BIGINT           NOT NULL UNIQUE,
    telegram_username TEXT             NOT NULL,
    status            inputting_status NOT NULL DEFAULT 'none'
);