CREATE TABLE instance (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    allow_add_friend INTEGER NOT NULL DEFAULT 1,
    allow_create_group INTEGER NOT NULL DEFAULT 1
);

INSERT INTO instance (id, allow_add_friend, allow_create_group) VALUES (1, 1, 1);
