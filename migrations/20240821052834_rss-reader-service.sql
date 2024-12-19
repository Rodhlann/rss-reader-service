CREATE TABLE IF NOT EXISTS categories (
  id serial PRIMARY KEY,
  name varchar NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS feeds (
  id serial PRIMARY KEY,
  name varchar NOT NULL UNIQUE,
  url varchar NOT NULL UNIQUE,
  category_id int NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
  created_date timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS cache (
  json jsonb
);
