CREATE TABLE IF NOT EXISTS categories (
  id serial PRIMARY KEY,
  name varchar NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS raw_feeds (
  id serial PRIMARY KEY,
  name varchar NOT NULL UNIQUE,
  url varchar NOT NULL UNIQUE,
  category_id int NOT NULL REFERENCES categories(id) ON DELETE RESTRICT,
  created_date timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS cached_feeds (
  id serial PRIMARY KEY,
  name varchar NOT NULL UNIQUE,
  category_id int NOT NULL REFERENCES categories(id) ON DELETE RESTRICT
  created_date timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS cached_entries (
  feed_id int NOT NULL REFERENCES cached_feeds(id) ON DELETE CASCADE,
  title text NOT NULL UNIQUE,
  url text NOT NULL UNIQUE,
  created_date timestamptz NOT NULL
);

CREATE INDEX cached_entries_feed_id_idx ON cached_entries(feed_id);
CREATE INDEX cached_entries_created_date_idx ON cached_entries(created_date);
