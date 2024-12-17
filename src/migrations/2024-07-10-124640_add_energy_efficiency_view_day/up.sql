-- Your SQL goes here

CREATE MATERIALIZED VIEW IF NOT EXISTS energy_efficiency_view_day
WITH (timescaledb.continuous) as
select
	eehh.machine_id,
    eehh.device_code,
    SUM(eehh.utilization_time) as utilization_time,
	SUM(eehh.consumption) AS consumption,
	time_bucket(INTERVAL '1 day', eehh.record_date) AS compilation_record_date
FROM
    energy_efficiency_hour_hist eehh
GROUP by
	compilation_record_date,
    eehh.device_code,
    eehh.machine_id
WITH NO DATA;

SELECT add_continuous_aggregate_policy('energy_efficiency_view_day',
  start_offset => INTERVAL '2 month',
  end_offset => INTERVAL '1 hour',
  schedule_interval => INTERVAL '1 day');
