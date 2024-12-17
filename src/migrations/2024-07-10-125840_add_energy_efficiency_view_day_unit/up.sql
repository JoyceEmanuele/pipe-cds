-- Your SQL goes here

CREATE MATERIALIZED VIEW IF NOT EXISTS energy_efficiency_view_day_unit
WITH (timescaledb.continuous) as
select
	m.unit_id,
	SUM(eehh.consumption) AS consumption,
	time_bucket(INTERVAL '1 day', eehh.record_date) AS compilation_record_date
FROM
    machines m
INNER JOIN 
    energy_efficiency_hour_hist eehh on m.id = eehh.machine_id
GROUP by
	compilation_record_date,
    m.unit_id
WITH NO DATA;

SELECT add_continuous_aggregate_policy('energy_efficiency_view_day_unit',
  start_offset => INTERVAL '2 month',
  end_offset => INTERVAL '1 hour',
  schedule_interval => INTERVAL '1 day');
