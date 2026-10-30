-- no-transaction
PRAGMA foreign_keys = OFF;

CREATE TABLE friend_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    friend_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE(user_id, friend_id),
    FOREIGN KEY (user_id) REFERENCES user(id) ON DELETE CASCADE,
    FOREIGN KEY (friend_id) REFERENCES user(id) ON DELETE CASCADE
);
INSERT INTO friend_new (id, user_id, friend_id, created_at)
SELECT id, user_id, friend_id, created_at FROM friend;
DROP TABLE friend;
ALTER TABLE friend_new RENAME TO friend;
CREATE INDEX idx_friend_user_id ON friend(user_id);

CREATE TABLE friend_request_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sender_id TEXT NOT NULL,
    receiver_id TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(sender_id, receiver_id),
    FOREIGN KEY (sender_id) REFERENCES user(id) ON DELETE CASCADE,
    FOREIGN KEY (receiver_id) REFERENCES user(id) ON DELETE CASCADE
);
INSERT INTO friend_request_new (id, sender_id, receiver_id, status, created_at, updated_at)
SELECT id, sender_id, receiver_id, status, created_at, updated_at FROM friend_request;
DROP TABLE friend_request;
ALTER TABLE friend_request_new RENAME TO friend_request;
CREATE INDEX idx_friend_request_receiver_id ON friend_request(receiver_id);
CREATE INDEX idx_friend_request_sender_id ON friend_request(sender_id);

CREATE TABLE file_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    owner_id TEXT NOT NULL,
    object_key TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    content_type TEXT NOT NULL,
    size INTEGER NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (owner_id) REFERENCES user(id) ON DELETE CASCADE
);
INSERT INTO file_new (id, owner_id, object_key, name, content_type, size, status, created_at)
SELECT id, owner_id, object_key, name, content_type, size, status, created_at FROM file;
DROP TABLE file;
ALTER TABLE file_new RENAME TO file;
CREATE INDEX idx_file_owner_id ON file(owner_id);

CREATE TABLE chat_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chat_type TEXT NOT NULL,
    name TEXT,
    owner_id TEXT,
    created_at TEXT NOT NULL,
    avatar INTEGER,
    FOREIGN KEY (owner_id) REFERENCES user(id) ON DELETE SET NULL,
    FOREIGN KEY (avatar) REFERENCES file(id) ON DELETE SET NULL
);
INSERT INTO chat_new (id, chat_type, name, owner_id, created_at, avatar)
SELECT id, chat_type, name, owner_id, created_at, avatar FROM chat;
DROP TABLE chat;
ALTER TABLE chat_new RENAME TO chat;
CREATE INDEX idx_chat_avatar ON chat(avatar);

CREATE TABLE chat_member_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chat_id INTEGER NOT NULL,
    user_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    last_read_message_id INTEGER,
    UNIQUE(chat_id, user_id),
    FOREIGN KEY (chat_id) REFERENCES chat(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES user(id) ON DELETE CASCADE
);
INSERT INTO chat_member_new (id, chat_id, user_id, created_at, last_read_message_id)
SELECT id, chat_id, user_id, created_at, last_read_message_id FROM chat_member;
DROP TABLE chat_member;
ALTER TABLE chat_member_new RENAME TO chat_member;
CREATE INDEX idx_chat_member_user_id ON chat_member(user_id);
CREATE INDEX idx_chat_member_chat_id ON chat_member(chat_id);

CREATE TABLE message_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chat_id INTEGER NOT NULL,
    sender_id TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    file_id INTEGER,
    FOREIGN KEY (chat_id) REFERENCES chat(id) ON DELETE CASCADE,
    FOREIGN KEY (sender_id) REFERENCES user(id) ON DELETE CASCADE,
    FOREIGN KEY (file_id) REFERENCES file(id) ON DELETE SET NULL
);
INSERT INTO message_new (id, chat_id, sender_id, content, created_at, file_id)
SELECT id, chat_id, sender_id, content, created_at, file_id FROM message;
DROP TABLE message;
ALTER TABLE message_new RENAME TO message;
CREATE INDEX idx_message_chat_id ON message(chat_id);
CREATE INDEX idx_message_chat_id_id ON message(chat_id, id);
CREATE INDEX idx_message_file_id ON message(file_id);

PRAGMA foreign_keys = ON;
