-- Your SQL goes here

CREATE MATERIALIZED VIEW IF NOT EXISTS water_hist_view
WITH (timescaledb.continuous) as
select
	unit_id,
    device_code,
	SUM(wh.consumption) AS consumption,
	time_bucket(INTERVAL '1 day', wh.record_date) AS compilation_record_date
FROM
    water_hist wh
GROUP by
	compilation_record_date,
    device_code,
    unit_id
WITH NO DATA;

SELECT add_continuous_aggregate_policy('water_hist_view',
  start_offset => INTERVAL '1 month',
  end_offset => INTERVAL '1 hour',
  schedule_interval => INTERVAL '1 day');
  