-- Users table. Emails are stored already-lowercased by the application, and the
-- UNIQUE constraint guarantees one account per address. Password hashes are
-- Argon2id PHC strings (they embed their own salt + parameters).
CREATE TABLE IF NOT EXISTS users (
    id            UUID PRIMARY KEY,
    email         TEXT        NOT NULL UNIQUE,
    password_hash TEXT        NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
