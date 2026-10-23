ALTER TABLE user ADD COLUMN avatar INTEGER REFERENCES file(id) ON DELETE SET NULL;

CREATE INDEX idx_user_avatar ON user(avatar);
