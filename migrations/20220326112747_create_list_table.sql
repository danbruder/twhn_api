-- Add migration script here

CREATE TABLE IF NOT EXISTS list (
    key TEXT NOT NULL,
    item_id INTEGER NOT NULL,
    ordering INTEGER NOT NULL,
    created_at DATETIME NOT NULL,
    PRIMARY KEY (key, item_id)
);
