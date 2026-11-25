ALTER TABLE reading ADD COLUMN opened_at INTEGER NOT NULL DEFAULT 0;
CREATE INDEX reading_recency ON reading(opened_at DESC,entry DESC);
PRAGMA user_version=3;
