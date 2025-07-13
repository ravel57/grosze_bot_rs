CREATE TABLE IF NOT EXISTS contacts
(
    id         SERIAL PRIMARY KEY,
    user_id    INTEGER NOT NULL REFERENCES users_t (id) ON DELETE CASCADE,
    contact_id INTEGER NOT NULL REFERENCES users_t (id) ON DELETE CASCADE,
    name       TEXT    NOT NULL,
    UNIQUE (user_id, contact_id)
);