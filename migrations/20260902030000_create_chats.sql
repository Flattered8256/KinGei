CREATE TABLE chat (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chat_type TEXT NOT NULL,
    name TEXT,
    owner_id TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (owner_id) REFERENCES user(id)
);

CREATE TABLE chat_member (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chat_id INTEGER NOT NULL,
    user_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE(chat_id, user_id),
    FOREIGN KEY (chat_id) REFERENCES chat(id),
    FOREIGN KEY (user_id) REFERENCES user(id)
);

CREATE INDEX idx_chat_member_user_id ON chat_member(user_id);
CREATE INDEX idx_chat_member_chat_id ON chat_member(chat_id);
