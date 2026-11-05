ALTER TABLE message ADD COLUMN reply_to_id BIGINT REFERENCES message(id) ON DELETE SET NULL;

CREATE INDEX idx_message_reply_to_id ON message(reply_to_id);
