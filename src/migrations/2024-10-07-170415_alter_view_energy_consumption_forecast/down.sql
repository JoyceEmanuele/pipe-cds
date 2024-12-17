-- This file should undo anything in `up.sql`
SELECT remove_continuous_aggregate_policy('energy_consumption_forecast_view');

SELECT add_continuous_aggregate_policy('energy_consumption_forecast_view',
  start_offset => INTERVAL '1 month',
  end_offset => INTERVAL '1 hour',
  schedule_interval => INTERVAL '1 day');
