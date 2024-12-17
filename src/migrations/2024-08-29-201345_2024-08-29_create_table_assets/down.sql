-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS assets;

ALTER TABLE machines DROP CONSTRAINT machines_uk_reference_id;