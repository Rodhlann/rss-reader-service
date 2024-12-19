-- Add migration script here
ALTER TABLE cache
ADD COLUMN name varchar NOT NULL UNIQUE,
ADD COLUMN created_date timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
ADD COLUMN xml_string text NOT NULL;

ALTER TABLE cache
DROP COLUMN json;