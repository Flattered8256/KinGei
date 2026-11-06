CREATE TABLE invite (
    id BIGINT PRIMARY KEY CHECK (id = 1),
    code TEXT NOT NULL
);
