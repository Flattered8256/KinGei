ALTER TABLE user ADD COLUMN role TEXT NOT NULL DEFAULT 'Member';

UPDATE user SET role = 'Admin' WHERE id = (
    SELECT id FROM user ORDER BY created_at ASC, id ASC LIMIT 1
);
