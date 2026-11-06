ALTER TABLE instance ADD COLUMN initializing INTEGER NOT NULL DEFAULT 1;

UPDATE instance SET initializing = 0 WHERE EXISTS (SELECT 1 FROM "user");
