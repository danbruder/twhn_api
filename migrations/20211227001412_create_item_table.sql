-- Add migration script here

CREATE TABLE IF NOT EXISTS item (
	id INTEGER PRIMARY KEY,
   	original TEXT NOT NULL
);
