-- Add migration script here

ALTER TABLE item ADD descendants INTEGER; -- story
ALTER TABLE item ADD username TEXT; --comment or story
ALTER TABLE item ADD score INTEGER; -- story, job
ALTER TABLE item ADD title TEXT; -- story , job 
ALTER TABLE item ADD url TEXT; -- story, job
ALTER TABLE item ADD body TEXT; -- comment or story, job
ALTER TABLE item ADD time INTEGER; -- comment or story


CREATE TABLE IF NOT EXISTS item_parent (
    item_id INTEGER NOT NULL,
    parent_id INTEGER NOT NULL,
    ordering INTEGER NOT NULL
);
