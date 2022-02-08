-- Add migration script here

CREATE TABLE IF NOT EXISTS bookmarked_item (
    item_id INTEGER NOT NULL,
    user_id TEXT NOT NULL,
    created_at DATETIME NOT NULL,
    PRIMARY KEY (item_id, user_id)
);
