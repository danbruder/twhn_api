-- Add migration script here

CREATE TABLE IF NOT EXISTS item (
	id INTEGER PRIMARY KEY,
   	original TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS item_metric (
    item_id INTEGER NOT NULL,
    metric TEXT NOT NULL,
    created_at DATETIME NOT NULL,
    value INTEGER NOT NULL,
    PRIMARY KEY (item_id, metric, created_at)
);
