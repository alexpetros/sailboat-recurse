CREATE TABLE profiles (
  profile_id INTEGER PRIMARY KEY,
  display_name TEXT NOT NULL,
  preferred_username TEXT NOT NULL,
  nickname TEXT,
  private_key_pem TEXT NOT NULL
) STRICT;

CREATE TABLE posts (
  post_id INTEGER PRIMARY KEY,
  profile_id INTEGER NOT NULL REFERENCES profiles ON DELETE CASCADE ON UPDATE CASCADE,
  content TEXT,
  created_at TEXT NOT NULL DEFAULT (strftime('%FT%TZ', CURRENT_TIMESTAMP))
) STRICT;

CREATE TABLE known_actors (
  actor_id TEXT PRIMARY KEY, -- note that this HAS to be a URL
  name TEXT NOT NULL,
  preferred_username TEXT NOT NULL,
  url TEXT,
  inbox TEXT,
  outbox TEXT,
  summary TEXT,
  icon_url TEXT
) STRICT;

CREATE TABLE followers (
  profile_id INTEGER NOT NULL REFERENCES profiles ON DELETE CASCADE ON UPDATE CASCADE,
  actor_id TEXT NOT NULL REFERENCES known_actors ON DELETE CASCADE ON UPDATE CASCADE
) STRICT;

CREATE TABLE following (
  profile_id INTEGER NOT NULL REFERENCES profiles ON DELETE CASCADE ON UPDATE CASCADE,
  actor_id TEXT NOT NULL REFERENCES known_actors ON DELETE CASCADE ON UPDATE CASCADE
) STRICT;

CREATE TABLE globals (
  key TEXT NOT NULL,
  value TEXT NOT NULL
) STRICT;

CREATE TABLE sessions (
  token TEXT NOT NULL UNIQUE,
  timestamp TEXT NOT NULL DEFAULT (strftime('%FT%TZ', CURRENT_TIMESTAMP))
) STRICT;

INSERT INTO globals (key, value) VALUES
('domain', 'sailboat.fly.dev');

/* INSERT INTO profiles (nickname, display_name, handle) */
/* VALUES ('Thoughts', 'Alex Petros', 'thoughts'); */

/* INSERT INTO posts (profile_id, content) */
/* VALUES (1, 'This is my first sailboat post!') */
/* ; */
