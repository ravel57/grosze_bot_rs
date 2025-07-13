CREATE TABLE IF NOT EXISTS users_t
(
    id                SERIAL PRIMARY KEY,
    telegram_id       BIGINT NOT NULL UNIQUE,
    telegram_username TEXT   NOT NULL
);