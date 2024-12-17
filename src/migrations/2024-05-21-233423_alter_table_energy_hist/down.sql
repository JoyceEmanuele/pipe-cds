-- This file should undo anything in `up.sql`
DROP MATERIALIZED VIEW IF EXISTS energy_hist_view;

ALTER TABLE energy_hist ALTER COLUMN consumption TYPE DECIMAL(10,2);

ALTER TABLE energy_hist DROP COLUMN is_measured_consumption;

ALTER TABLE energy_hist DROP COLUMN is_valid_consumption;

CREATE MATERIALIZED VIEW IF NOT EXISTS energy_hist_view
WITH (timescaledb.continuous) as
select
	ec.unit_id AS unit_id,
	SUM(eh.consumption) AS consumption,
	time_bucket(INTERVAL '1 day', eh.record_date) AS compilation_record_date
FROM
    energy_hist eh
JOIN
    electric_circuits ec ON eh.electric_circuit_id  = ec.id
GROUP by
	compilation_record_date,
    ec.unit_id
WITH NO DATA;

SELECT add_continuous_aggregate_policy('energy_hist_view',
  start_offset => INTERVAL '1 month',
  end_offset => INTERVAL '1 hour',
  schedule_interval => INTERVAL '1 day');

