CREATE TABLE media (
    file_id INTEGER PRIMARY KEY REFERENCES file(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    duration_ms INTEGER,
    thumbnail_id INTEGER REFERENCES file(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_media_thumbnail_id ON media(thumbnail_id);
