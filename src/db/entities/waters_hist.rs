use chrono::{NaiveDateTime, NaiveDate};
use diesel::{prelude::*, sql_query};
use diesel::sql_types::{Array, Date, Integer, Text};
use diesel::upsert::excluded;
use rust_decimal::Decimal;
use crate::models::database_models::waters_hist::WatersHist;
use crate::models::database_models::water_hist::WaterHist;
use crate::schedules::scheduler::write_to_log_file_thread;
use crate::GlobalVars;
use crate::schema::{self, waters_hist, water_hist};
use crate::http::structs::water::{GetLastValidConsumption, GetWaterDayGraphicInfoResponse, GetWaterGraphicInfoResponse, GetWaterConsumption, GetWaterUsageHistoryResponse, GetWaterUsageRequestBody, GetWaterUsageResponse, GetWaterYearUsageRequestBody, GetWaterYearUsageResponse};
use std::sync::Arc;
use std::error::Error;


pub fn insert_data_waters(data: WatersHist, globs: &Arc<GlobalVars>) -> Result<(), Box<dyn Error>> {
    let mut pool = globs.pool.get()?;

    let result = diesel::insert_into(schema::waters_hist::table)
        .values(&data)
        .execute(&mut pool);

    match result {
        Ok(rows_affected) => {}
        Err(err) => {
            write_to_log_file_thread(&format!("Error inserting data in waters_hist, {:?}", err), 0, "ERROR");
            eprintln!("Inserting data error: {:?}, {}", data, err);
        }
    }

    drop(pool);

    Ok(())
}

pub fn insert_data_water(data: WaterHist, globs: &Arc<GlobalVars>) -> Result<(), Box<dyn Error>> {
    let mut pool = globs.pool.get()?;
    
    let result = diesel::insert_into(water_hist::table)
        .values(&data)
        .on_conflict((water_hist::unit_id, water_hist::record_date))
        .do_update()
        .set((
            water_hist::consumption.eq(excluded(water_hist::consumption)),
            water_hist::is_measured_consumption.eq(excluded(water_hist::is_measured_consumption)),
            water_hist::is_valid_consumption.eq(excluded(water_hist::is_valid_consumption)),
        ))
        .execute(&mut pool);
    
    match result {
        Ok(rows_affected) => {}
        Err(err) => {
            write_to_log_file_thread(&format!("Error inserting data in water_hist, {:?}", err), 0, "ERROR");
            eprintln!("Inserting data error: {:?}, {}", data, err);
        }
    }

    drop(pool);

    Ok(())
}

pub fn get_water_month_usage(params: GetWaterUsageRequestBody, globs: &Arc<GlobalVars>) -> Result<Vec<GetWaterUsageResponse>, Box<dyn Error>> {

    let mut pool = globs.pool.get()?;

    let sqlQuery = sql_query("
    SELECT
        water_hist_view.unit_id,
        water_hist_view.device_code,
        SUM(water_hist_view.consumption) AS consumption,
        time_bucket(INTERVAL '1 month', water_hist_view.compilation_record_date) AS compilation_record_date
    FROM
        water_hist_view 
    INNER JOIN units ON (units.id = water_hist_view.unit_id)
    WHERE
        units.reference_id IN (SELECT unnest($1::integer[])) AND
        compilation_record_date >= to_timestamp($2, 'YYYY-MM-DD') AND
        compilation_record_date <= to_timestamp($3, 'YYYY-MM-DD')
    GROUP BY
        compilation_record_date,
        water_hist_view.device_code,
        water_hist_view.unit_id
    "); 

    let sqlQuery = sqlQuery
    .bind::<Array<Integer>, _>(params.unitIds)
    .bind::<Text, _>(params.startDate.clone())
    .bind::<Text, _>(params.endDate.clone());    

    let response = sqlQuery.load::<GetWaterUsageResponse>(&mut pool)?;

    Ok(response)
}

pub fn get_water_dates_year_usage(params: GetWaterUsageRequestBody, globs: &Arc<GlobalVars>) -> Result<Vec<GetWaterUsageResponse>, Box<dyn Error>> {
    let mut pool = globs.pool.get()?;

    let sqlQuery = sql_query("
    SELECT DISTINCT ON (compilation_record_date)
        water_hist_view.unit_id,
        water_hist_view.device_code,
        water_hist_view.consumption,
        time_bucket(INTERVAL '1 year', water_hist_view.compilation_record_date) AS compilation_record_date
    FROM
        water_hist_view 
    INNER JOIN units ON (units.id = water_hist_view.unit_id)
    WHERE
        units.reference_id IN (SELECT unnest($1::integer[])) AND
        compilation_record_date >= to_timestamp($2, 'YYYY-MM-DD') AND
        compilation_record_date <= to_timestamp($3, 'YYYY-MM-DD') AND
        consumption > 0
    GROUP BY
        compilation_record_date,
        water_hist_view.unit_id,
        water_hist_view.device_code,
        water_hist_view.consumption
    "); 

    let sqlQuery = sqlQuery
    .bind::<Array<Integer>, _>(params.unitIds)
    .bind::<Text, _>(params.startDate.clone())
    .bind::<Text, _>(params.endDate.clone());    

    let response = sqlQuery.load::<GetWaterUsageResponse>(&mut pool)?;

    Ok(response)
}

pub fn get_water_year_usage(params: GetWaterYearUsageRequestBody, globs: &Arc<GlobalVars>) -> Result<Vec<GetWaterYearUsageResponse>, Box<dyn Error>> {
    let mut pool = globs.pool.get()?;

    let sqlQuery = sql_query("
    SELECT
        water_hist_view.unit_id,
        water_hist_view.device_code,
        SUM(water_hist_view.consumption) AS consumption,
        time_bucket(INTERVAL '1 year', water_hist_view.compilation_record_date) AS compilation_record_date
    FROM
        water_hist_view 
    INNER JOIN units ON (units.id = water_hist_view.unit_id)
    WHERE
        units.reference_id IN (SELECT unnest($1::integer[])) AND
        compilation_record_date >= to_timestamp($2, 'YYYY-MM-DD') AND
        compilation_record_date <= to_timestamp($3, 'YYYY-MM-DD')
    GROUP BY
        compilation_record_date,
        water_hist_view.device_code,
        water_hist_view.unit_id
    "); 

    let sqlQuery = sqlQuery
    .bind::<Array<Integer>, _>(params.unitIds)
    .bind::<Text, _>(params.startDate.clone())
    .bind::<Text, _>(params.endDate.clone());    

    let response = sqlQuery.load::<GetWaterYearUsageResponse>(&mut pool)?;

    Ok(response)
}

pub fn get_last_valid_consumption(unit_id: i32, record_date: NaiveDateTime, globs: &Arc<GlobalVars>) -> Result<Option<GetLastValidConsumption>, Box<dyn Error>> {
    let mut pool = globs.pool.get()?;

    let result: Option<(rust_decimal::Decimal, NaiveDateTime)> = schema::water_hist::table
        .filter(schema::water_hist::unit_id.eq(unit_id))
        .filter(water_hist::record_date.lt(record_date))
        .filter(water_hist::is_measured_consumption.eq(false))
        .filter(water_hist::is_valid_consumption.eq(true))
        .select((schema::water_hist::consumption, schema::water_hist::record_date))
        .order(schema::water_hist::record_date.desc())
        .first(&mut pool)
        .optional()?;

        match result {
            Some((consumption, record_date)) => {
                Ok(Some(GetLastValidConsumption {
                    consumption,
                    record_date,
                }))
            },
            None => Ok(None),
        }

}

pub fn delete_data_waters_hist(unit_id: i32, production_timestamp: &str, globs: &Arc<GlobalVars>) -> Result<(), Box<dyn Error>> {
    let mut pool = globs.pool.get()?;

    let parsed_date = NaiveDate::parse_from_str(production_timestamp, "%Y-%m-%dT%H:%M:%S%.3fZ")
        .map_err(|e| format!("Failed to parse production_timestamp: {}", e))?;

    diesel::delete(schema::waters_hist::table
        .filter(schema::waters_hist::unit_id.eq(unit_id))
        .filter(schema::waters_hist::record_date.lt(parsed_date)))
        .execute(&mut pool)?;

    drop(pool);

    Ok(())
} 

pub fn get_hour_usage_history(unit_id: i32, start_date: &str, end_date: &str, globs: &Arc<GlobalVars>) -> Result<Vec<GetWaterUsageHistoryResponse>, Box<dyn Error>> {
    let mut pool = globs.pool.get()?;
    
    let water_usage_history = sql_query("
    SELECT
        wh.device_code,
        wh.consumption as usage,
        wh.record_date as information_date
    FROM
        water_hist wh
        INNER JOIN units u on (u.id = wh.unit_id)
    WHERE
        u.reference_id = $1 and
        wh.record_date BETWEEN to_timestamp($2, 'YYYY-MM-DD HH24:MI:SS') AND to_timestamp($3, 'YYYY-MM-DD HH24:MI:SS')
    ORDER BY 
        wh.record_date ASC
    ");

    let water_usage_history = water_usage_history
        .bind::<Integer, _>(unit_id)
        .bind::<Text, _>(start_date)
        .bind::<Text, _>(end_date);

    let response = water_usage_history.load::<GetWaterUsageHistoryResponse>(&mut pool)?;
    
    Ok(response)
}

pub fn get_day_usage_history(unit_id: i32, start_date: &str, end_date: &str, globs: &Arc<GlobalVars>) -> Result<Vec<GetWaterUsageHistoryResponse>, Box<dyn Error>> {
    let mut pool = globs.pool.get()?;
    
    let water_usage_history = sql_query("
    SELECT
        whv.device_code,
        whv.consumption as usage,
        whv.compilation_record_date as information_date
    FROM
        water_hist_view whv
        INNER JOIN units u on (u.id = whv.unit_id)
    WHERE
        u.reference_id = $1 and
        whv.compilation_record_date BETWEEN to_timestamp($2, 'YYYY-MM-DD HH24:MI:SS') AND to_timestamp($3, 'YYYY-MM-DD HH24:MI:SS')
    ORDER BY 
        whv.compilation_record_date ASC
    ");

    let water_usage_history = water_usage_history
        .bind::<Integer, _>(unit_id)
        .bind::<Text, _>(start_date)
        .bind::<Text, _>(end_date);

    let response = water_usage_history.load::<GetWaterUsageHistoryResponse>(&mut pool)?;
    
    Ok(response)
}

pub fn get_year_usage_history(unit_id: i32, start_date: &str, end_date: &str, globs: &Arc<GlobalVars>) -> Result<Vec<GetWaterUsageHistoryResponse>, Box<dyn Error>> {
    let mut pool = globs.pool.get()?;
    
    let water_usage_history = sql_query("
    SELECT
        whv.device_code,
        time_bucket(INTERVAL '1 month', whv.compilation_record_date) as information_date,
        SUM(whv.consumption) as usage
    FROM
        water_hist_view whv
        INNER JOIN units u on (u.id = whv.unit_id)
    WHERE
        u.reference_id = $1 and
        whv.compilation_record_date BETWEEN to_timestamp($2, 'YYYY-MM-DD HH24:MI:SS') AND to_timestamp($3, 'YYYY-MM-DD HH24:MI:SS')
    GROUP BY
        information_date,
        whv.device_code
    ");

    let water_usage_history = water_usage_history
        .bind::<Integer, _>(unit_id)
        .bind::<Text, _>(start_date)
        .bind::<Text, _>(end_date);

    let response = water_usage_history.load::<GetWaterUsageHistoryResponse>(&mut pool)?;
    
    Ok(response)
}

pub fn get_water_info_by_hour_graphic(unit_id: i32, start_date: &str, end_date: &str, globs: &Arc<GlobalVars>) -> Result<Option<GetWaterConsumption>, Box<dyn Error>> {
    let mut pool: diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>> = globs.pool.get()?;

    let water_hist = sql_query("
    SELECT
        SUM(wh.consumption) as consumption
    FROM 
        water_hist wh
        INNER JOIN units u on (u.id = wh.unit_id)
    WHERE
        wh.record_date  >= to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS') AND
        wh.record_date  <= to_timestamp($2, 'YYYY-MM-DD HH24:MI:SS') AND
        u.reference_id = $3
    HAVING
        SUM(wh.consumption) IS NOT NULL
    ");

    let water_hist = water_hist
    .bind::<Text, _>(start_date)
    .bind::<Text, _>(end_date)
    .bind::<Integer, _>(unit_id);

    let response = water_hist.get_result::<GetWaterConsumption>(&mut pool).optional()?;

    Ok(response)
}

pub fn get_water_info_by_day_graphic(unit_id: i32, start_date: &str, end_date: &str, globs: &Arc<GlobalVars>) -> Result<Option<GetWaterDayGraphicInfoResponse>, Box<dyn Error>> {
    let mut pool: diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>> = globs.pool.get()?;

    let water_hist = sql_query("
    SELECT
        SUM(whv.consumption) as consumption,
        ROUND(AVG(whv.consumption), 2) AS average_consumption,
        COUNT(*) as qtd_consumption
    FROM 
        water_hist_view whv
        INNER JOIN units u on (u.id = whv.unit_id)
    where
        whv.compilation_record_date  >= to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS') AND
        whv.compilation_record_date  <= to_timestamp($2, 'YYYY-MM-DD HH24:MI:SS') AND
        u.reference_id = $3
    HAVING
        SUM(whv.consumption) IS NOT NULL
    ");

    let water_hist = water_hist
    .bind::<Text, _>(start_date)
    .bind::<Text, _>(end_date)
    .bind::<Integer, _>(unit_id);

    let response = water_hist.get_result::<GetWaterDayGraphicInfoResponse>(&mut pool).optional()?;


    Ok(response)
}

pub fn get_water_consumption_in_dates(unit_id: i32, dates: Vec<NaiveDate>, globs: &Arc<GlobalVars>)-> Result<GetWaterConsumption, Box<dyn Error>> {
    let mut pool: diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>> = globs.pool.get()?;

    let water_consumption = sql_query("
    SELECT
        COALESCE(SUM(wh.consumption), 0) as consumption
    FROM
        water_hist wh
    WHERE
        wh.unit_id = $1 and
        DATE(wh.record_date) = ANY($2::date[])
    ");

    let water_consumption = water_consumption
        .bind::<Integer, _>(unit_id)
        .bind::<Array<Date>, _>(dates);

    let response: GetWaterConsumption = water_consumption.get_result(&mut pool).unwrap_or_else(|_| {
        GetWaterConsumption { consumption: Decimal::new(0, 0) }
    });

    Ok(response)
}
