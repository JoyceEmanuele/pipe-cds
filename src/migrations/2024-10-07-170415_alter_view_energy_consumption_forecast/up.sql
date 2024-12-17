-- Your SQL goes here
SELECT remove_continuous_aggregate_policy('energy_consumption_forecast_view');

SELECT add_continuous_aggregate_policy('energy_consumption_forecast_view',
  start_offset => INTERVAL '1 month',
  end_offset => INTERVAL '-1 month',
  schedule_interval => INTERVAL '1 day');

