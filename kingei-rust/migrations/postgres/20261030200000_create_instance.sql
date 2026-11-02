CREATE TABLE instance (
    id BIGINT PRIMARY KEY CHECK (id = 1),
    allow_add_friend BOOLEAN NOT NULL DEFAULT TRUE,
    allow_create_group BOOLEAN NOT NULL DEFAULT TRUE
);

INSERT INTO instance (id, allow_add_friend, allow_create_group) VALUES (1, TRUE, TRUE);
