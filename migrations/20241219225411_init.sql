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
);

CREATE TABLE IF NOT EXISTS entries (
  feed_id int NOT NULL REFERENCES cached_feeds(id) ON DELETE CASCADE,
  title text NOT NULL UNIQUE,
  url text NOT NULL UNIQUE,
  created_date timestamptz NOT NULL
);

CREATE INDEX entries_feed_id_idx ON entries(feed_id);
CREATE INDEX entries_created_date_idx ON entries(created_date);
