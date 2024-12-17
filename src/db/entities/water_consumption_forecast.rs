use diesel::{prelude::*, sql_query};
use diesel::sql_types::{Integer, Text};
use crate::http::structs::water::GetWaterForecastUsageResponse;
use crate::models::database_models::water_consumption_forecast::WaterConsumptionForecast;
use crate::schedules::scheduler::write_to_log_file_thread;
use crate::schema::water_consumption_forecast;
use crate::{schema, GlobalVars};
use std::sync::Arc;
use std::error::Error;

pub fn insert_update_water_consumption_forecast(data: WaterConsumptionForecast, globs: &Arc<GlobalVars>) -> Result<(), Box<dyn Error>> {
    let mut pool = globs.pool.get()?;

    let result = diesel::insert_into(schema::water_consumption_forecast::table)
        .values(&data)
        .on_conflict((water_consumption_forecast::unit_id, water_consumption_forecast::forecast_date))
        .do_update()
        .set((
            data.monday.map(|value|  water_consumption_forecast::monday.eq(value)),
            data.tuesday.map(|value| water_consumption_forecast::tuesday.eq(value)),
            data.wednesday.map(|value| water_consumption_forecast::wednesday.eq(value)),
            data.thursday.map(|value| water_consumption_forecast::thursday.eq(value)),
            data.friday.map(|value| water_consumption_forecast::friday.eq(value)),
            data.saturday.map(|value| water_consumption_forecast::saturday.eq(value)),
            data.sunday.map(|value| water_consumption_forecast::sunday.eq(value)),
        ))
        .execute(&mut pool);

    match result {
        Ok(_) => {}
        Err(err) => {
            write_to_log_file_thread(&format!("Error inserting data in water_consumption_forecast, {:?}", err), 0, "ERROR");
            eprintln!("Inserting data error: {:?}, {}", data, err);
        }
    }

    drop(pool);

    Ok(())
}

pub fn get_forecast_usage(unit_id: i32, forecast_date: &str, globs: &Arc<GlobalVars>) -> Result<Option<GetWaterForecastUsageResponse>, Box<dyn Error>>{
    let mut pool = globs.pool.get()?;

    let forecast_usage = sql_query("
    select
        coalesce(current_month.monday, previous_month.monday, 0) as monday,
        coalesce(current_month.tuesday, previous_month.tuesday, 0) as tuesday,
        coalesce(current_month.wednesday, previous_month.wednesday, 0) as wednesday,
        coalesce(current_month.thursday, previous_month.thursday, 0) as thursday,
        coalesce(current_month.friday, previous_month.friday, 0) as friday,
        coalesce(current_month.saturday, previous_month.saturday, 0) as saturday,
        coalesce(current_month.sunday, previous_month.sunday, 0) as sunday
    from
        (select 
            wcf.monday,
            wcf.tuesday,
            wcf.wednesday,
            wcf.thursday,
            wcf.friday,
            wcf.saturday,
            wcf.sunday
        from 
            water_consumption_forecast wcf
            inner join units u on u.id = wcf.unit_id
        where
            u.reference_id = $1 and
            wcf.forecast_date = DATE($2)
        ) as current_month
    full outer join 
        (select 
            wcf.monday,
            wcf.tuesday,
            wcf.wednesday,
            wcf.thursday,
            wcf.friday,
            wcf.saturday,
            wcf.sunday
        from 
            water_consumption_forecast wcf
            inner join units u on u.id = wcf.unit_id
        where
            u.reference_id = $1 and
            wcf.forecast_date = DATE_TRUNC('month', DATE($2)) - interval '1 month'
        ) as previous_month
    on 1=1
    ");

    let forecast_usage = forecast_usage
    .bind::<Integer, _>(unit_id)
    .bind::<Text, _>(forecast_date);

    let response = forecast_usage.get_result::<GetWaterForecastUsageResponse>(&mut pool).optional()?;
    
    Ok(response)
}
