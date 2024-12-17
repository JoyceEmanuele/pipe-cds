-- Your SQL goes here

CREATE MATERIALIZED VIEW IF NOT EXISTS energy_consumption_forecast_view
WITH (timescaledb.continuous) as
select
	ec.unit_id AS unit_id,
	SUM(ecf.consumption_forecast) AS consumption_forecast,
	time_bucket(INTERVAL '1 day', ecf.record_date) AS compilation_record_date
FROM
    energy_consumption_forecast ecf
JOIN
    electric_circuits ec ON ecf.electric_circuit_id  = ec.id
GROUP by
	compilation_record_date,
    ec.unit_id
WITH NO DATA;

SELECT add_continuous_aggregate_policy('energy_consumption_forecast_view',
  start_offset => INTERVAL '1 month',
  end_offset => INTERVAL '1 hour',
  schedule_interval => INTERVAL '1 day');
