-- This file should undo anything in `up.sql`
ALTER MATERIALIZED VIEW energy_hist_view_month set (timescaledb.materialized_only = true); 
