-- This file should undo anything in `up.sql`
DROP MATERIALIZED VIEW IF EXISTS chiller_parameters_hist_view;

DROP TABLE IF EXISTS chiller_parameters_minutes_hist;
