-- Add migration script here

CREATE TABLE IF NOT EXISTS config (
	key TEXT PRIMARY KEY,
   	value TEXT NOT NULL
);
