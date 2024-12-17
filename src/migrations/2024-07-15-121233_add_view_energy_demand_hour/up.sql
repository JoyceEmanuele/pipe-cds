-- Your SQL goes here
CREATE MATERIALIZED VIEW IF NOT EXISTS energy_demand_hist_view
WITH (timescaledb.continuous) AS
SELECT
    demh.electric_circuit_id,
    time_bucket('1 hour', demh.record_date) AS compilation_record_date,
    ROUND(AVG(demh.average_demand), 2) AS average_demand,
    MAX(demh.max_demand) AS max_demand,
    MIN(demh.min_demand) AS min_demand
FROM
    energy_demand_minutes_hist demh
GROUP BY
    compilation_record_date,
    demh.electric_circuit_id
WITH NO DATA;

SELECT add_continuous_aggregate_policy('energy_demand_hist_view',
  start_offset => INTERVAL '2 month',
  end_offset => INTERVAL '1 hour',
  schedule_interval => INTERVAL '1 hour');
