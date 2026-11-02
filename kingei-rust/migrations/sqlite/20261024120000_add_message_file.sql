ALTER TABLE message ADD COLUMN file_id INTEGER REFERENCES file(id) ON DELETE SET NULL;

CREATE INDEX idx_message_file_id ON message(file_id);
