-- Your SQL goes here
CREATE MATERIALIZED VIEW IF NOT EXISTS unit_disponibility_hist
WITH (timescaledb.continuous) as
select
	u.id AS unit_id,
    ROUND(AVG(ddh.disponibility), 2) AS average_disponibility,
	time_bucket(INTERVAL '1 day', ddh.record_date) AS compilation_record_date
FROM
    device_disponibility_hist ddh
INNER JOIN
    units u ON u.id  = ddh.unit_id
GROUP by
	compilation_record_date,
    u.id
WITH NO DATA;

SELECT add_continuous_aggregate_policy('unit_disponibility_hist',
  start_offset => INTERVAL '2 month',
  end_offset => INTERVAL '1 hour',
  schedule_interval => INTERVAL '1 day');