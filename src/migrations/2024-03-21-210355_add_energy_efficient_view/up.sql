-- Your SQL goes here

CREATE MATERIALIZED VIEW IF NOT EXISTS energy_efficiency_view
WITH (timescaledb.continuous) as
select
	m.unit_id  as unit_id,
	SUM(ee.consumption) AS refrigeration_consumption,
	SUM(case 
			when ee.capacity_power < 100 then ee.capacity_power
			else ee.capacity_power / 12000
		end
		) as capacity_power,
	time_bucket(INTERVAL '1 day', ee.record_date) AS compilation_record_date
FROM
    machines m
JOIN
    energy_efficiency_hist ee ON m.id = ee.machine_id 
GROUP by
	compilation_record_date,
    m.unit_id
WITH NO DATA;

SELECT add_continuous_aggregate_policy('energy_efficiency_view',
  start_offset => INTERVAL '1 month',
  end_offset => INTERVAL '1 hour',
  schedule_interval => INTERVAL '1 day');