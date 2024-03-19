CREATE TABLE posts (
  post_id INTEGER PRIMARY KEY,
  feed_id INTEGER NOT NULL REFERENCES feeds ON DELETE CASCADE ON UPDATE CASCADE,
  content TEXT,
  created_at INTEGER NOT NULL DEFAULT (unixepoch())
) STRICT;

CREATE TABLE feeds (
  feed_id INTEGER PRIMARY KEY,
  internal_name TEXT NOT NULL,
  display_name TEXT NOT NULL,
  handle TEXT NOT NULL
) STRICT;

CREATE TABLE globals (
  key TEXT NOT NULL,
  value TEXT NOT NULL
) STRICT;

INSERT INTO globals (key, value) VALUES
('domain', 'example.com');

INSERT INTO feeds (internal_name, display_name, handle)
VALUES ('Thoughts', 'Alex Petros', 'thoughts');

INSERT INTO posts (feed_id, content)
VALUES (1, 'This is my first sailboat post!')
;
