CREATE TABLE posts (
  post_id INTEGER PRIMARY KEY,
  profile_id INTEGER NOT NULL REFERENCES profiles ON DELETE CASCADE ON UPDATE CASCADE,
  content TEXT,
  created_at INTEGER NOT NULL DEFAULT (unixepoch())
) STRICT;

CREATE TABLE profiles (
  profile_id INTEGER PRIMARY KEY,
  internal_name TEXT NOT NULL,
  display_name TEXT NOT NULL,
  handle TEXT NOT NULL,
  private_key_pem TEXT NOT NULL
) STRICT;

CREATE TABLE followed_actors (
  followed_actor_id INTEGER PRIMARY key,
  name TEXT NOT NULL,
  handle TEXT NOT NULL,
  host TEXT NOT NULL,
  url TEXT NOT NULL UNIQUE,
  summary TEXT
) STRICT;

CREATE TABLE globals (
  key TEXT NOT NULL,
  value TEXT NOT NULL
) STRICT;

INSERT INTO globals (key, value) VALUES
('domain', 'sailboat.fly.dev');

/* INSERT INTO profiles (internal_name, display_name, handle) */
/* VALUES ('Thoughts', 'Alex Petros', 'thoughts'); */

/* INSERT INTO posts (profile_id, content) */
/* VALUES (1, 'This is my first sailboat post!') */
/* ; */
