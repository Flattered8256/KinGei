ALTER TABLE chat_member ADD COLUMN last_read_message_id INTEGER;

CREATE TABLE message (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chat_id INTEGER NOT NULL,
    sender_id TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (chat_id) REFERENCES chat(id),
    FOREIGN KEY (sender_id) REFERENCES user(id)
);

CREATE INDEX idx_message_chat_id ON message(chat_id);
CREATE INDEX idx_message_chat_id_id ON message(chat_id, id);
