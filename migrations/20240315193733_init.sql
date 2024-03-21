-- Add migration script here
-- Created by `sqlx migrate add init` command
CREATE TABLE IF NOT EXISTS todos
(
    id          INTEGER PRIMARY KEY NOT NULL,
    title       TEXT                NOT NULL,
    notes       TEXT                NOT NULL DEFAULT 'note',
    completed   BOOLEAN             NOT NULL DEFAULT 0
);