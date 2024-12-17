-- Your SQL goes here
CREATE MATERIALIZED VIEW IF NOT EXISTS energy_hist_view_month
WITH (timescaledb.continuous) as
select
	ec.unit_id AS unit_id,
	SUM(eh.consumption) AS consumption,
	time_bucket(INTERVAL '1 month', eh.record_date) AS compilation_record_date
FROM
    energy_hist eh
JOIN
    electric_circuits ec ON eh.electric_circuit_id  = ec.id
GROUP by
	compilation_record_date,
    ec.unit_id
WITH NO DATA;

SELECT add_continuous_aggregate_policy('energy_hist_view_month',
  start_offset => INTERVAL '5 year',
  end_offset => INTERVAL '1 hour',
  schedule_interval => INTERVAL '1 day');