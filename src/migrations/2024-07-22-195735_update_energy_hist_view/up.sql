-- Your SQL goes here

drop materialized view IF EXISTS energy_hist_view;

CREATE MATERIALIZED VIEW IF NOT EXISTS energy_hist_view
WITH (timescaledb.continuous) as
select
	ec.unit_id AS unit_id,
	SUM(case when eh.is_valid_consumption then eh.consumption else 0 end) AS consumption,
	COUNT(CASE WHEN eh.is_valid_consumption = false THEN 1 END) AS invalid_consumption_count,
	(MAX(eh.is_measured_consumption::int) = 1) as contains_processed,
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