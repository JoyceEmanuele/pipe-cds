-- This file should undo anything in `up.sql`
drop materialized view IF EXISTS energy_hist_view_month;

CREATE MATERIALIZED VIEW IF NOT EXISTS energy_hist_view_month
WITH (timescaledb.continuous) as
select
	ec.unit_id AS unit_id,
	case
		when MIN(case when eh.is_valid_consumption then 1 else 0 end) = 1 THEN SUM(eh.consumption)
	END AS consumption,
	case
		when MIN(case when eh.is_valid_consumption then 1 else 0 end) = 0 then TRUE else FALSE
	end as contains_invalid,
	case 
		when MAX(case when eh.is_measured_consumption  then 1 else 0 end) = 1 then TRUE else FALSE
	end as contains_processed,
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

ALTER MATERIALIZED VIEW energy_hist_view_month set (timescaledb.materialized_only = false); 
