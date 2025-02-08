CREATE TABLE IF NOT EXISTS categories (
  id serial PRIMARY KEY,
  name varchar NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS raw_feeds (
  id serial PRIMARY KEY,
  name varchar NOT NULL UNIQUE,
  url varchar NOT NULL UNIQUE,
  category_id int NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
  created_date timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS cached_feeds (
  id serial PRIMARY KEY,
  name varchar NOT NULL UNIQUE,
  category_id int NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
  created_date timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS cached_entries (
  id serial PRIMARY KEY,
  feed_id int NOT NULL REFERENCES cached_feeds(id) ON DELETE CASCADE,
  title text NOT NULL,
  url text NOT NULL UNIQUE,
  created_date timestamptz NOT NULL
);

CREATE INDEX cached_entries_feed_id_idx ON cached_entries(feed_id);
CREATE INDEX cached_entries_created_date_idx ON cached_entries(created_date);

CREATE OR REPLACE FUNCTION delete_orphaned_categories()
RETURNS TRIGGER AS $$
BEGIN
    DELETE FROM categories
    WHERE id NOT IN (SELECT category_id FROM raw_feeds)
        AND id NOT IN (SELECT category_id FROM cached_feeds);
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_delete_orphaned_categories
AFTER UPDATE OR DELETE ON raw_feeds
FOR EACH STATEMENT
EXECUTE FUNCTION delete_orphaned_categories();

CREATE TRIGGER trigger_delete_orphaned_categories
AFTER UPDATE OR DELETE ON cached_feeds
FOR EACH STATEMENT
EXECUTE FUNCTION delete_orphaned_categories();
