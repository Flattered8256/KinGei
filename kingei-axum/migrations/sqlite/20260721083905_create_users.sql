-- Add migration script here
CREATE TABLE user (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL,
    password_hash TEXT NOT NULL
);