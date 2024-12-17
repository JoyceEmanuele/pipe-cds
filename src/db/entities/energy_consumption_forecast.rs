use chrono::{Datelike, NaiveDate};
use diesel::sql_types::{Array, Integer, Nullable, Numeric, Text};
use diesel::{prelude::*, sql_query};
use diesel::upsert::excluded;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::http::structs::energy::{EnergyTrends, GetEnergyAnalysisHistFilterRequestBody, GetEnergyAnalysisHistFilterResponse, GetEnergyTrendsRequestBody, GetEnergyTrendsResponse, GetMonthlyTargetSQL, GetTrendsSQL};
use crate::models::database_models::energy_consumption_forecast::EnergyConsumptionForecast;
use crate::schedules::scheduler::write_to_log_file_thread;
use crate::schema::energy_consumption_forecast;
use crate::GlobalVars;
use std::sync::Arc;
use std::error::Error;

#[derive(Deserialize, Clone)]
pub struct GetEnergyTarget {
    pub startDate: String,
    pub endDate: String,
    pub electCircuitIds: Vec<i32>,
}

#[derive(QueryableByName, Serialize, Clone, Copy, Debug)]
pub struct GetEnergyTargetResponse {
    #[diesel(sql_type = Numeric)]
    pub consumption_target: Decimal,
}

pub fn insert_data_energy_consumption_forecast(data: EnergyConsumptionForecast, globs: &Arc<GlobalVars>) -> Result<(), Box<dyn Error>> {
    let mut pool = globs.pool.get()?;
    
    let result = diesel::insert_into(energy_consumption_forecast::table)
        .values(&data)
        .on_conflict((energy_consumption_forecast::electric_circuit_id, energy_consumption_forecast::record_date))
        .do_update()
        .set((
            energy_consumption_forecast::consumption_forecast.eq(excluded(energy_consumption_forecast::consumption_forecast)),
        ))
        .execute(&mut pool);

    match result {
        Ok(rows_affected) => {}
        Err(err) => {
            write_to_log_file_thread(&format!("Error inserting data in energy_consumption_forecast, {:?}", err), 0, "ERROR");
            eprintln!("Inserting data error: {:?}, {}", data, err);
        }
    }
    
    drop(pool);
    
    Ok(())
}

pub fn get_energy_consumption_target(params: GetEnergyTarget, globs: &Arc<GlobalVars>) -> Result<(GetEnergyTargetResponse), Box<dyn Error>> {
    let mut pool = globs.pool.get()?;

    let mut sql = "
        SELECT 
            COALESCE(SUM(ecf.consumption_forecast), 0) as consumption_target
        FROM
            energy_consumption_forecast ecf
            INNER JOIN electric_circuits ec on ec.id = ecf.electric_circuit_id
        WHERE
            ecf.record_date between to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS') and to_timestamp($2, 'YYYY-MM-DD HH24:MI:SS') + interval '1 month' - interval '1 day'
            AND ec.reference_id = ANY($3::integer[])".to_string();

    let sqlQuery = sql_query(sql)
        .bind::<Text, _>(params.startDate.clone())
        .bind::<Text, _>(params.endDate.clone())
        .bind::<Array<Integer>, _>(params.electCircuitIds.clone());

    let response = sqlQuery.load::<GetEnergyTargetResponse>(&mut pool)?;

    drop(pool);
        
    Ok(response[0])
}

pub fn get_months_with_energy_consumtion_forecast(params: GetEnergyAnalysisHistFilterRequestBody, globs: &Arc<GlobalVars>) -> Result<(Vec<GetEnergyAnalysisHistFilterResponse>), Box<dyn Error>> {

    let mut pool = globs.pool.get()?;

    let mut unitsFilterSQL = String::new();

    if !params.units.is_empty() {
        unitsFilterSQL = " AND u.reference_id = ANY($1::integer[]) ".to_string();
    }
    let mut sql = format!("
        SELECT
            time_bucket(INTERVAL '1 month', compilation_record_date) AS time
        FROM 
            energy_consumption_forecast_view ecfv
            JOIN units u on ecfv.unit_id = u.id
        WHERE
            ecfv.compilation_record_date <= to_timestamp($2, 'YYYY-MM-DD')
            {}
        GROUP BY
            time
        ORDER BY
            time;", unitsFilterSQL);

    let sqlQuery = sql_query(sql)
        .bind::<Nullable<Array<Integer>>, _>(params.units)
        .bind::<Text, _>(params.date.clone());


   let response = sqlQuery.load::<GetEnergyAnalysisHistFilterResponse>(&mut pool)?;

    Ok(response)
}

pub fn energy_trends(params: &GetEnergyTrendsRequestBody, globs: &Arc<GlobalVars>) -> Result<(GetEnergyTrendsResponse), Box<dyn Error>> {
    let mut pool = globs.pool.get()?;

    let mut unitsFilterSQL = String::new();

    if !params.units.is_empty() {
        unitsFilterSQL = " AND u.reference_id = ANY($1::integer[]) ".to_string();
    }

    let mut sql = format!("
    SELECT
        ROUND(SUM(ehv.consumption), 2) AS consumption,
        ROUND(COALESCE(SUM(ecfv.consumption_forecast), 0), 2) AS forecast,
        COALESCE(ehv.compilation_record_date, ecfv.compilation_record_date) AS time
    FROM
        energy_hist_view ehv
        FULL OUTER JOIN energy_consumption_forecast_view ecfv on ehv.unit_id = ecfv.unit_id and ehv.compilation_record_date = ecfv.compilation_record_date 
        INNER JOIN units u ON COALESCE(ehv.unit_id, ecfv.unit_id) = u.id  
    WHERE
        COALESCE(ehv.compilation_record_date, ecfv.compilation_record_date) BETWEEN to_timestamp($2, 'YYYY-MM-DD') AND to_timestamp($3, 'YYYY-MM-DD')
        {}
    GROUP BY
        time
    ORDER BY
        time ASC", unitsFilterSQL
    );

    let sqlQuery = sql_query(sql)
    .bind::<Nullable<Array<Integer>>, _>(params.units.clone())
        .bind::<Text, _>(params.startDate.clone())
        .bind::<Text, _>(params.endDate.clone());

   let response = sqlQuery.load::<GetTrendsSQL>( &mut pool)?;


    let mut sqlConsumptionTarget = format!("
        SELECT 
            SUM(emct.consumption_target) as target
        FROM
            units u 
            JOIN energy_monthly_consumption_target emct on emct.unit_id = u.id
        WHERE 
            emct.date_forecast = to_timestamp($2, 'YYYY-MM-DD')
            {}", unitsFilterSQL);

    let sqlQueryConsumptionTarget = sql_query(sqlConsumptionTarget)
        .bind::<Nullable<Array<Integer>>, _>(params.units.clone())
        .bind::<Text, _>(params.startDate.clone());


    let responseConsumptionTarget = sqlQueryConsumptionTarget.load::<GetMonthlyTargetSQL>(&mut pool)?;

    drop(pool);

    let start_date = NaiveDate::parse_from_str(&params.startDate, "%Y-%m-%d").unwrap();
    let end_date = NaiveDate::parse_from_str(&params.endDate, "%Y-%m-%d").unwrap();
    let total_days = (end_date - start_date).num_days() + 1;
    let today = NaiveDate::from_ymd(chrono::Local::now().year(), chrono::Local::now().month(), chrono::Local::now().day());

    if !response.is_empty() {
        let mut consumption: Decimal = Decimal::new(0, 0);
        let mut consumptionTarget: Option<Decimal> = None;
        let mut consumptionDayTarget: Option<Decimal> = None;
        let mut consumptionTargetActual: Option<Decimal> = None;
        let mut trendsData: Vec<EnergyTrends> = Vec::new();
        let mut consumption_forecast_aux = Decimal::new(0, 0);

        if responseConsumptionTarget[0].target.is_some() {
            consumptionDayTarget = Some(responseConsumptionTarget[0].target.unwrap_or(Decimal::new(0,0)) / Decimal::new(total_days,0));
        };

        for day in response.iter() {
            consumptionTarget = match consumptionDayTarget {
                Some(target) => Some(consumptionTarget.unwrap_or(Decimal::new(0,0)) + target.clone()),
                _ => None
            };

            consumptionTargetActual = match (day.consumption, consumptionDayTarget) {
                (Some(_), Some(target)) => Some(consumptionTargetActual.unwrap_or(Decimal::new(0,0)) + target.clone()),
                _ => consumptionTargetActual
            };

            consumption = match day.consumption {
                Some(dayConsumption) => dayConsumption.clone() + consumption,
                _ => consumption
            };

            let is_after_or_same_today = today <= day.time.clone().date();

            if is_after_or_same_today {
                consumption_forecast_aux += day.forecast.clone();
            } else {
                consumption_forecast_aux = consumption.clone();
            }


            trendsData.push(EnergyTrends {
                time: day.time.clone(),
                consumption: if is_after_or_same_today { Decimal::new(0, 0) } else { consumption.clone() },
                consumptionForecast: consumption_forecast_aux,
                consumptionTarget: consumptionTarget.clone(),
                consumptionOverTarget: if is_after_or_same_today {
                    Decimal::new(0, 0)
                } else {
                    match consumptionTarget {
                        Some(target) => (consumption - target).max(Decimal::new(0, 0)),
                        None => Decimal::new(0, 0),
                    }
                },
                consumptionPercentage: match consumptionTarget {
                    Some(target) => {
                        if is_after_or_same_today {
                            ((consumption_forecast_aux) / target) * Decimal::new(100, 0)
                        } else {
                            ((consumption) / target) * Decimal::new(100, 0)
                        }
                    }
                    None => Decimal::new(0, 0),
                }
            });
        }

        return Ok(GetEnergyTrendsResponse {
            trendsData,
            monthlyForecast: Some(consumption_forecast_aux),
            monthlyTarget: responseConsumptionTarget[0].target,
            totalConsumption: Some(consumption),
            totalConsumtionPercentage: match (responseConsumptionTarget[0].target, consumption > Decimal::new(0, 0)) {
                (Some(monthlyTarget), true) => Some((consumption / monthlyTarget) * Decimal::new(100, 0)),
                _ => Some(Decimal::new(0, 0)),
            },
            monthlyForecastPercentage: match responseConsumptionTarget[0].target {
                Some(target) => Some((consumption_forecast_aux / target) * Decimal::new(100, 0)),
                _ => Some(Decimal::new(0, 0))
            }
        });
    }

    Ok(GetEnergyTrendsResponse {
        trendsData: Vec::new(),
        monthlyForecast: None,
        monthlyTarget: None,
        totalConsumption: None,
        monthlyForecastPercentage: None,
        totalConsumtionPercentage: None
    })
}

pub fn process_energy_forecast_view(start_date: &str, end_date: &str, globs: &Arc<GlobalVars>) ->Result<(), Box<dyn Error>> {
    let mut pool = globs.pool.get()?;

    let query = format!(
        "CALL refresh_continuous_aggregate('energy_consumption_forecast_view', '{} 00:00:00'::timestamp, '{} 23:59:59'::timestamp);",
        start_date, end_date
    );
    
    sql_query(query).execute(&mut pool)?;
    
    drop(pool);
    
    Ok(())
}
