ALTER TABLE chat ADD COLUMN avatar INTEGER REFERENCES file(id) ON DELETE SET NULL;

CREATE INDEX idx_chat_avatar ON chat(avatar);
