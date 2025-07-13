CREATE TABLE IF NOT EXISTS transactions
(
    id           SERIAL PRIMARY KEY,
    from_user_id INTEGER NOT NULL REFERENCES users_t (id) ON DELETE SET NULL,
    to_user_id   INTEGER NOT NULL REFERENCES users_t (id) ON DELETE SET NULL,
    amount       NUMERIC NOT NULL
);