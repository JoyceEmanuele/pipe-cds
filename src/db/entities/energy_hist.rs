use chrono::NaiveDateTime;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::sql_types::{Array, Integer, Nullable, Numeric, Text};
use rust_decimal::{self, Decimal};
use rust_decimal::prelude::ToPrimitive;
use diesel::upsert::excluded;
use diesel::{prelude::*, sql_query};
use serde::{Deserialize, Serialize};
use crate::http::structs::energy::{GetDayEnergyConsumptionResponse, GetEnergyAnalysisHistFilterRequestBody, GetEnergyAnalysisHistFilterResponse, GetEnergyAnalysisHistRequestBody, GetEnergyAnalysisHistResponse, GetEnergyAnalysisListRequestBody, GetEnergyAnalysisListResponse, GetEnergyAnalysisListResponseSQL, GetEnergyConsumptionResponse, GetGeneralUnitsStats, GetHourEnergyConsumptionResponse, GetLastValidConsumption, GetProcelInsightsRequestBody, GetProcelInsigthsResponse, GetTotalDaysConsumptionUnit, GetTotalUnitsWithConsumption, GetUnitConsumptionByArea, GetUnitEnergyStats, GetUnitListProcelRequestBody, GetUnitListRequestBody, GetUnitListResponse, OrderByTypeEnum, ParamsGetTotalDaysConsumptionUnit, ProcelType};
use crate::http::structs::energy::{GetEnergyAnalysisHistResponseWithFlags, GetEnergyAnalysisListResponseWithFlags};
use crate::models::database_models::energy_hist::EnergyHist;
use crate::schedules::scheduler::write_to_log_file_thread;
use crate::schema::{self, electric_circuits, energy_hist};
use crate::GlobalVars;
use std::sync::Arc;
use std::error::Error;

#[derive(Deserialize, Clone)]
pub struct GetEnergyAverage {
    pub date: String,
    pub electCircuitId: i32,
}

#[derive(QueryableByName, Serialize, Clone, Copy)]
pub struct GetEnergyAverageResponse {
    #[diesel(sql_type = Numeric)]
    pub consumption_average: Decimal,
}

pub fn insert_data_energy(data: EnergyHist, globs: &Arc<GlobalVars>) -> Result<(), Box<dyn Error>> {
    let mut pool = globs.pool.get()?;
    
    let result = diesel::insert_into(energy_hist::table)
        .values(&data)
        .on_conflict((energy_hist::electric_circuit_id, energy_hist::record_date))
        .do_update()
        .set((
            energy_hist::consumption.eq(excluded(energy_hist::consumption)),
            energy_hist::is_measured_consumption.eq(excluded(energy_hist::is_measured_consumption)),
            energy_hist::is_valid_consumption.eq(excluded(energy_hist::is_valid_consumption)),
        ))
        .execute(&mut pool);

    match result {
        Ok(rows_affected) => {}
        Err(err) => {
            write_to_log_file_thread(&format!("Error inserting data in energy_hist, {:?}", err), 0, "ERROR");
            eprintln!("Inserting data error: {:?}, {}", data, err);
        }
    }
    
    drop(pool);
    
    Ok(())
}


pub fn apply_energy_flags_to_unit_list(params: Vec<GetEnergyAnalysisListResponseSQL>, isDielUser: bool) -> Result<(Vec<GetEnergyAnalysisListResponseWithFlags>), Box<dyn Error>> {

    let mut response: Vec<GetEnergyAnalysisListResponseWithFlags> = Vec::new();

    for unit in params.iter(){

        let mut responseUnit: GetEnergyAnalysisListResponseWithFlags = GetEnergyAnalysisListResponseWithFlags {
            client_name: unit.client_name.clone(),
            reference_id: unit.reference_id.clone(),
            unit_name: unit.unit_name.clone(),
            state_name: unit.state_name.clone(),
            city_name: unit.city_name.clone(),
            capacity_power: unit.capacity_power.clone(),
            consumption: unit.consumption.clone(),
            consumption_by_area: unit.consumption_by_area.clone(),
            refrigeration_consumption: unit.refrigeration_consumption.clone(),
            refrigeration_consumption_by_area: unit.refrigeration_consumption_by_area.clone(),
            refrigeration_consumption_percentage: unit.refrigeration_consumption_percentage.clone(),
            total_charged: unit.total_charged.clone(),
            ranking: unit.ranking.clone(),
            category: unit.category.clone(),
            previous_consumption: unit.previous_consumption.clone(),
            invalid: false,
            processed: false
        };

        match isDielUser {
            false => {
                if (unit.invalid_count > (unit.readings_count * Decimal::new(10, 2))) || (unit.processed_count > (unit.readings_count * Decimal::new(10, 2))) {
                    if unit.processed_count > unit.invalid_count {
                        responseUnit.processed = true;
                    } else {
                        responseUnit.invalid = true;
                    }

                    response.push(responseUnit);
                } else {
                    response.push(responseUnit);
                }
            },
            true => {
                if (unit.invalid_count > Decimal::new(0, 0)) || (unit.processed_count > Decimal::new(0, 0)) {
                    if unit.processed_count > unit.invalid_count {
                        responseUnit.processed = true;
                    } else {
                        responseUnit.invalid = true;
                    }

                    response.push(responseUnit);
                } else {
                    response.push(responseUnit);
                }
            }
        };
    }

    Ok(response)
}

pub fn get_energy_consumption_by_days(params: GetEnergyAnalysisHistRequestBody, globs: &Arc<GlobalVars>) -> Result<(Vec<GetEnergyAnalysisHistResponse>), Box<dyn Error>> {
    let mut conn = PgConnection::establish(&globs.configfile.POSTGRES_DATABASE_URL).expect("CONNECTION DB ERROR: ");

    let mut unitsFilterSQL = String::new();

    if !params.units.is_empty() {
        unitsFilterSQL = " units.reference_id = ANY($1::integer[]) AND ".to_string();
    }
    
    let mut sql = format!("
        SELECT
            ROUND(SUM(ehv.consumption), 2) AS consumption,
            ROUND(SUM(ehv.consumption * units.tarifa_kwh), 2) AS total_charged,
            SUM(ehv.invalid_consumption_count) AS invalid_count,
            SUM(ehv.processed_consumption_count) AS processed_count,
            SUM(ehv.readings_count) AS readings_count,
            ehv.compilation_record_date AS time,
            COALESCE(COUNT(DISTINCT ehv.unit_id), 0) AS units_count
        FROM 
            energy_hist_view ehv
        JOIN 
            units ON ehv.unit_id = units.id
        WHERE
            {}
            ehv.compilation_record_date BETWEEN to_timestamp($2, 'YYYY-MM-DD') AND to_timestamp($3, 'YYYY-MM-DD')
        GROUP BY 
            ehv.compilation_record_date
        ORDER BY 
            time;", unitsFilterSQL);

    let sqlQuery = sql_query(sql)
        .bind::<Array<Integer>, _>(params.units.clone())
        .bind::<Text, _>(params.startDate.clone())
        .bind::<Text, _>(params.endDate.clone());

   let response = sqlQuery.load::<GetEnergyAnalysisHistResponse>(&mut conn)?;
        
    Ok(response)
}

pub fn get_energy_consumption_by_months(params: GetEnergyAnalysisHistRequestBody, globs: &Arc<GlobalVars>) -> Result<(Vec<GetEnergyAnalysisHistResponse>), Box<dyn Error>> {
    let mut conn = PgConnection::establish(&globs.configfile.POSTGRES_DATABASE_URL).expect("CONNECTION DB ERROR: ");
    
    let mut unitsFilterSQL = String::new();

    if !params.units.is_empty() {
        unitsFilterSQL = " units.reference_id = ANY($1::integer[]) AND ".to_string();
    }

    let mut sql = format!("
        SELECT
            ROUND(SUM(ehvm.consumption), 2) AS consumption,
            ROUND(SUM(ehvm.consumption * units.tarifa_kwh), 2) AS total_charged,
            SUM(ehvm.invalid_consumption_count) AS invalid_count,
            SUM(ehvm.processed_consumption_count) AS processed_count,
            SUM(ehvm.readings_count) AS readings_count,
            ehvm.compilation_record_date AS time,
            COALESCE(COUNT(DISTINCT ehvm.unit_id), 0) AS units_count
        FROM 
            energy_hist_view_month ehvm
        JOIN 
            units ON ehvm.unit_id = units.id
        WHERE
            {}
            ehvm.compilation_record_date BETWEEN to_timestamp($2, 'YYYY-MM-DD') AND to_timestamp($3, 'YYYY-MM-DD')
        GROUP BY 
            ehvm.compilation_record_date
        ORDER BY 
            time;", unitsFilterSQL);


    let sqlQuery = sql_query(sql)
        .bind::<Nullable<Array<Integer>>, _>(params.units.clone())
        .bind::<Text, _>(params.startDate.clone())
        .bind::<Text, _>(params.endDate.clone());;

   let response = sqlQuery.load::<GetEnergyAnalysisHistResponse>(&mut conn)?;
        
    Ok(response)
}

pub fn get_total_units_with_consumption(params: GetEnergyAnalysisHistRequestBody, globs: &Arc<GlobalVars>) -> Result<GetTotalUnitsWithConsumption, Box<dyn Error>> {
    let mut conn = PgConnection::establish(&globs.configfile.POSTGRES_DATABASE_URL).expect("CONNECTION DB ERROR: ");

    let mut filter_unit_ids = String::new();

    if !params.units.is_empty() {
        filter_unit_ids = " units.reference_id = ANY($1::integer[]) AND ".to_string();
    }
    
    let mut sql = format!("
        SELECT
            COALESCE(COUNT(DISTINCT ehv.unit_id), 0) AS units_count
        FROM 
            energy_hist_view ehv
        INNER JOIN 
            units ON ehv.unit_id = units.id
        WHERE
            {}
            ehv.compilation_record_date BETWEEN to_timestamp($2, 'YYYY-MM-DD') AND to_timestamp($3, 'YYYY-MM-DD')
        ", filter_unit_ids);

    let sqlQuery = sql_query(sql)
        .bind::<Array<Integer>, _>(params.units.clone())
        .bind::<Text, _>(params.startDate.clone())
        .bind::<Text, _>(params.endDate.clone());

   let response = sqlQuery.get_result::<GetTotalUnitsWithConsumption>(&mut conn)?;
        
    Ok(response)
}

pub fn get_total_units_with_constructed_area(params: GetEnergyAnalysisHistRequestBody, globs: &Arc<GlobalVars>) -> Result<GetTotalUnitsWithConsumption, Box<dyn Error>> {
    let mut conn = PgConnection::establish(&globs.configfile.POSTGRES_DATABASE_URL).expect("CONNECTION DB ERROR: ");

    let mut filter_unit_ids = String::new();

    if !params.units.is_empty() {
        filter_unit_ids = " units.reference_id = ANY($1::integer[]) AND ".to_string();
    }
    
    let mut sql = format!("
        SELECT 
		    COUNT(*) AS units_count
		FROM (
		    SELECT 
		        ehv.unit_id
		    FROM 
		        energy_hist_view ehv
		    INNER JOIN 
		        units ON ehv.unit_id = units.id
		    WHERE
                {}
		        ehv.compilation_record_date BETWEEN to_timestamp($2, 'YYYY-MM-DD') AND to_timestamp($3, 'YYYY-MM-DD') AND
		        units.constructed_area IS NOT NULL
		    GROUP BY 
		        ehv.unit_id
		    HAVING 
		        SUM(ehv.consumption) > $4
		) AS filtered_units
        ", filter_unit_ids);

    let sqlQuery = sql_query(sql)
        .bind::<Array<Integer>, _>(params.units.clone())
        .bind::<Text, _>(params.startDate.clone())
        .bind::<Text, _>(params.endDate.clone())
        .bind::<Integer, _>(params.minConsumption.clone());


   let response = sqlQuery.get_result::<GetTotalUnitsWithConsumption>(&mut conn)?;
        
    Ok(response)
}

pub fn apply_energy_flags_to_unit_by_time(params: Vec<GetEnergyAnalysisHistResponse>, isDielUser: bool) -> Result<(Vec<GetEnergyAnalysisHistResponseWithFlags>), Box<dyn Error>> {
    let mut response: Vec<GetEnergyAnalysisHistResponseWithFlags> = Vec::new();

    for time in params.iter(){
        let mut responseTime: GetEnergyAnalysisHistResponseWithFlags = GetEnergyAnalysisHistResponseWithFlags {
            time: time.time.clone(),
            consumption: time.consumption.clone(),
            total_charged: time.total_charged.clone(),
            invalid: false,
            processed: false,
            units_count: time.units_count,
        };

        match isDielUser {
            false => {
                if (time.invalid_count > (time.readings_count * Decimal::new(10, 2))) || (time.processed_count > (time.readings_count * Decimal::new(10, 2))) {
                    if time.processed_count > time.invalid_count {
                        responseTime.processed = true;
                    } else {
                        responseTime.invalid = true;
                    }

                    response.push(responseTime);
                } else {
                    response.push(responseTime);
                }
            },
            true => {
                if (time.invalid_count > Decimal::new(0, 0)) || (time.processed_count > Decimal::new(0, 0)) {
                    if time.processed_count > time.invalid_count {
                        responseTime.processed = true;
                    } else {
                        responseTime.invalid = true;
                    }

                    response.push(responseTime);
                } else {
                    response.push(responseTime);
                }
            }
        };
    }

    Ok(response)
}

pub fn get_months_with_energy_consumtion(params: GetEnergyAnalysisHistFilterRequestBody, globs: &Arc<GlobalVars>) -> Result<(Vec<GetEnergyAnalysisHistFilterResponse>), Box<dyn Error>> {

    let mut conn = PgConnection::establish(&globs.configfile.POSTGRES_DATABASE_URL).expect("CONNECTION DB ERROR: ");

    let mut unitsFilterSQL = String::new();

    if !params.units.is_empty() {
        unitsFilterSQL = " units.reference_id = ANY($1::integer[]) and ".to_string();
    }
    
    let mut sql = format!("
    SELECT
        compilation_record_date as time
    FROM energy_hist_view_month ehvm
        inner join units on ehvm.unit_id = units.id
    WHERE
        {}
        ehvm.compilation_record_date <= to_timestamp($2, 'YYYY-MM-DD')
    GROUP BY
        time
    ORDER BY
        time;", unitsFilterSQL);

    let sqlQuery = sql_query(sql)
        .bind::<Nullable<Array<Integer>>, _>(params.units)
        .bind::<Text, _>(params.date.clone());
    

   let response = sqlQuery.load::<GetEnergyAnalysisHistFilterResponse>(&mut conn)?;
        
    Ok(response)
}

pub fn get_energy_day_consumption(unit_id: i32, start_date: &str, end_date: &str, globs: &Arc<GlobalVars>) -> Result<Vec<GetDayEnergyConsumptionResponse>, Box<dyn Error>> {
    let mut pool = globs.pool.get()?;

    let energy_hist = sql_query("
        SELECT 
            TO_CHAR(time_bucket(INTERVAL '1 day', record_date), 'YYYY-MM-DD') AS day,
            SUM(energy_hist.consumption) as total_measured,
            MAX(energy_hist.consumption) as max_day_total_measured,
            electric_circuits.reference_id as electric_circuit_reference_id,
            COUNT(CASE WHEN energy_hist.is_valid_consumption = false then 1 end) AS invalid_count,
            COUNT(CASE WHEN energy_hist.is_measured_consumption = true then 1 end) AS processed_count,
            COUNT(*) as readings_count
        FROM
            energy_hist
            INNER JOIN electric_circuits on (electric_circuits.id = energy_hist.electric_circuit_id)
            INNER JOIN units on (units.id = electric_circuits.unit_id)
        WHERE
            units.reference_id = $1 AND
            energy_hist.record_date >= to_timestamp($2, 'YYYY-MM-DD HH24:MI:SS') AND
            energy_hist.record_date <= to_timestamp($3, 'YYYY-MM-DD HH24:MI:SS')
        GROUP BY
            day, electric_circuit_reference_id");

    let energy_hist = energy_hist
        .bind::<Integer, _>(unit_id)
        .bind::<Text, _>(start_date)
        .bind::<Text, _>(end_date);

    let response = energy_hist.load::<GetDayEnergyConsumptionResponse>(&mut pool)?;

    Ok(response)
}

pub fn get_energy_hours_consumption(unit_id: i32, start_date: &str, end_date: &str, globs: &Arc<GlobalVars>) -> Result<Vec<GetHourEnergyConsumptionResponse>, Box<dyn Error>> {
    let mut pool = globs.pool.get()?;
    
    let energy_hist = sql_query("
    SELECT 
        time_bucket(INTERVAL '1 hour', record_date) AS hour,
        energy_hist.consumption as total_measured,
        electric_circuits.reference_id as electric_circuit_reference_id,
        case
			when min(case when energy_hist.is_valid_consumption then 1 else 0 end) = 0 then TRUE else FALSE
	    end as contains_invalid,
	    case 
		    when MAX(case when energy_hist.is_measured_consumption then 1 else 0 end) = 1 then TRUE else FALSE
	    end as contains_processed
    FROM
        energy_hist
        INNER JOIN electric_circuits on (electric_circuits.id = energy_hist.electric_circuit_id)
        INNER JOIN units on (units.id = electric_circuits.unit_id)
    WHERE
        units.reference_id = $1 AND
        energy_hist.record_date >= to_timestamp($2, 'YYYY-MM-DD HH24:MI:SS') AND
        energy_hist.record_date <= to_timestamp($3, 'YYYY-MM-DD HH24:MI:SS')
    GROUP BY
        energy_hist.record_date, energy_hist.consumption, electric_circuits.reference_id
    ORDER BY
        hour ASC
    ");

    let energy_hist = energy_hist
        .bind::<Integer, _>(unit_id)
        .bind::<Text, _>(start_date)
        .bind::<Text, _>(end_date);

    let response = energy_hist.load::<GetHourEnergyConsumptionResponse>(&mut pool)?;
    
    Ok(response)
}

pub fn get_last_valid_consumption(electric_circuit_id: i32, record_date: NaiveDateTime, globs: &Arc<GlobalVars>) -> Result<Option<GetLastValidConsumption>, Box<dyn Error>> {
    let mut pool = globs.pool.get()?;

    let result: Option<(rust_decimal::Decimal, NaiveDateTime)> = schema::energy_hist::table
        .filter(schema::energy_hist::electric_circuit_id.eq(electric_circuit_id))
        .filter(energy_hist::record_date.lt(record_date))
        .filter(energy_hist::is_measured_consumption.eq(false))
        .filter(energy_hist::is_valid_consumption.eq(true))
        .select((schema::energy_hist::consumption, schema::energy_hist::record_date))
        .order(schema::energy_hist::record_date.desc())
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

pub fn delete_data_energy_hist(unit_id: i32, production_timestamp: &str, globs: &Arc<GlobalVars>) -> Result<(), Box<dyn Error>> {
    let mut pool = globs.pool.get()?;
    let parsed_timestamp = NaiveDateTime::parse_from_str(production_timestamp, "%Y-%m-%dT%H:%M:%S%.3fZ")
        .map_err(|e| format!("Failed to parse production_timestamp: {}", e))?;

     diesel::delete(energy_hist::table
        .filter(energy_hist::electric_circuit_id.eq_any(
            electric_circuits::table
                .select(electric_circuits::id)
                .filter(electric_circuits::unit_id.eq(unit_id))
        ))
        .filter(energy_hist::record_date.lt(parsed_timestamp)))
        .execute(&mut pool)?;

    drop(pool);

    Ok(())
} 

pub fn get_energy_units(params: &GetUnitListRequestBody, globs: &Arc<GlobalVars>) -> Result<(Vec<GetUnitListResponse>), Box<dyn Error>> {
    let mut conn = PgConnection::establish(&globs.configfile.POSTGRES_DATABASE_URL).expect("CONNECTION DB ERROR: ");

    let mut unitFilter = String::new();

    if !params.units.is_empty() {
        unitFilter = " AND u.reference_id = ANY($1::integer[]) ".to_string();
    };

    let mut sql = format!("
        SELECT
            u.reference_id AS unit_id,
            u.city_name AS city_name,
            u.state_name AS state_name
        FROM
            energy_hist_view ehv
            JOIN
                units u ON ehv.unit_id = u.id
        WHERE
            ehv.compilation_record_date BETWEEN to_timestamp($2, 'YYYY-MM-DD') AND to_timestamp($3, 'YYYY-MM-DD')
            {}
        GROUP BY
            u.reference_id, u.city_name, u.state_name", unitFilter);

    let sqlQuery = sql_query(sql)
        .bind::<Array<Integer>, _>(params.units.clone())
        .bind::<Text, _>(params.startDate.clone())
        .bind::<Text, _>(params.endDate.clone());

    let response = sqlQuery.load::<GetUnitListResponse>(&mut conn)?;
        
    Ok(response)
}

pub fn get_energy_analysis_list(params: &GetEnergyAnalysisListRequestBody, globs: &Arc<GlobalVars>) -> Result<(GetEnergyAnalysisListResponse), Box<dyn Error>> {
    let mut conn = PgConnection::establish(&globs.configfile.POSTGRES_DATABASE_URL).expect("CONNECTION DB ERROR: ");

    let mut unitFilter = String::new();

    if !params.units.is_empty() {
        unitFilter = " AND u.reference_id = ANY($1::integer[]) ".to_string();
    }
    
    let mut sql = format!("
        with area_consumption AS (
            SELECT
                energy_hist_view.unit_id as unit_id,
                ROW_NUMBER() OVER( 
                    ORDER BY (SUM(energy_hist_view.consumption) / u.constructed_area ) Asc
                ) AS ranking
            FROM
                energy_hist_view
                join units u on u.id = energy_hist_view.unit_id
            WHERE
                u.constructed_area IS NOT NULL
                AND u.constructed_area > 0
                AND energy_hist_view.compilation_record_date BETWEEN to_timestamp($2, 'YYYY-MM-DD') AND to_timestamp($3, 'YYYY-MM-DD')
                {}
            GROUP BY
                energy_hist_view.unit_id, u.constructed_area
            HAVING
                SUM(energy_hist_view.consumption) >= $13
        ), 
        previous_consumption AS (
            SELECT
                energy_hist_view.unit_id as unit_id,
                SUM(energy_hist_view.consumption) AS consumption
            FROM
                energy_hist_view
                join units u on u.id = energy_hist_view.unit_id
            WHERE
                energy_hist_view.compilation_record_date BETWEEN to_timestamp($14, 'YYYY-MM-DD') AND to_timestamp($15, 'YYYY-MM-DD')
                {}
            GROUP BY
                energy_hist_view.unit_id
        )
        SELECT
            u.reference_id AS reference_id,
            u.unit_name AS unit_name,
            c.client_name AS client_name,
            u.city_name AS city_name,
            u.state_name AS state_name,
            u.capacity_power AS capacity_power,
            ROUND(SUM(ehv.consumption), 2) AS consumption,
            ROUND(SUM(efv.consumption), 2) AS refrigeration_consumption,
            SUM(ehv.invalid_consumption_count) AS invalid_count,
            SUM(ehv.processed_consumption_count) AS processed_count,
            SUM(ehv.readings_count) AS readings_count,
            ac.ranking as ranking,
            CASE
                WHEN pc.consumption IS NULL THEN 0
                WHEN pc.consumption = 0 THEN 0
                ELSE ROUND((((SUM(ehv.consumption) - pc.consumption) / pc.consumption) * 100), 2)
            END AS previous_consumption,
            CASE
                WHEN u.reference_id = ANY($6::integer[]) THEN 'A'
                WHEN u.reference_id = ANY($7::integer[]) THEN 'B'
                WHEN u.reference_id = ANY($8::integer[]) THEN 'C'
                WHEN u.reference_id = ANY($9::integer[]) THEN 'D'
                WHEN u.reference_id = ANY($10::integer[]) THEN 'E'
                WHEN u.reference_id = ANY($11::integer[]) THEN 'F'
                WHEN u.reference_id = ANY($12::integer[]) THEN 'G'
                ELSE NULL
            end category,
            CASE
                WHEN SUM(efv.consumption) <= 0 THEN NULL
                WHEN SUM(ehv.consumption) <= 0 THEN NULL
                ELSE ROUND((SUM(efv.consumption) * 100.0) / (SUM(ehv.consumption)), 2)
            END AS refrigeration_consumption_percentage,
            CASE
                WHEN u.constructed_area = 0 THEN NULL
                ELSE ROUND((SUM(ehv.consumption) / u.constructed_area), 2)
            END AS consumption_by_area,
            CASE
                WHEN u.capacity_power = 0 THEN NULL
                ELSE ROUND(u.constructed_area / u.capacity_power, 2)
            END AS refrigeration_consumption_by_area,
            CASE
                WHEN u.tarifa_kwh = 0 THEN NULL
                ELSE ROUND((SUM(ehv.consumption) * u.tarifa_kwh), 2)
            END AS total_charged
        FROM
            energy_hist_view ehv
            LEFT JOIN
                energy_efficiency_view_day_unit efv ON ehv.unit_id = efv.unit_id 
                AND efv.compilation_record_date = ehv.compilation_record_date
            JOIN
                units u ON ehv.unit_id = u.id
            JOIN
                clients c ON u.client_id = c.id
            LEFT JOIN
                area_consumption ac ON ac.unit_id = ehv.unit_id
            LEFT JOIN
                previous_consumption pc ON pc.unit_id = ehv.unit_id
        WHERE
            ehv.compilation_record_date BETWEEN to_timestamp($2, 'YYYY-MM-DD') AND to_timestamp($3, 'YYYY-MM-DD')
            {}
        GROUP BY
            u.reference_id, u.unit_name, c.client_name, u.city_name, u.state_name, u.constructed_area, u.tarifa_kwh,
            u.capacity_power, ac.ranking, pc.consumption", unitFilter, unitFilter, 
                match &params.categoryFilter {
                    Some(category) => 
                        match category.len() > 0 {
                            true => " AND u.reference_id = ANY($16::integer[]) ".to_string(),
                            false => unitFilter.clone()
                        },
                    None => unitFilter.clone()
                }
    );

    let orderField = match &params.orderByField {
        Some(order) => order,
        _ => "ranking"
    };

    let mut orderByType = String::new();

    if(params.orderByType.is_none()){
        orderByType = "Asc".to_string();
    } else {
        orderByType = match &params.orderByType.clone().unwrap() {
            OrderByTypeEnum::Asc => "Asc".to_string(),
            OrderByTypeEnum::Desc => "Desc".to_string()
        };
    }

    sql = format!("{}{}", sql, match orderField {
        "unitName" => format!("{}{}", " ORDER BY unit_name ", orderByType),
        "consumption" => format!("{}{}", " ORDER BY consumption ", orderByType),
        "refrigerationConsumption" => format!("{}{}", " ORDER BY refrigeration_consumption ", orderByType),
        "refCapacity" => format!("{}{}", " ORDER BY capacity_power ", orderByType),
        "refrigerationConsumptionPercentage" => format!("{}{}{}", " ORDER BY refrigeration_consumption_percentage ", orderByType, " NULLS LAST"),
        "consumptionByArea" => format!("{}{}{}", " ORDER BY consumption_by_area ", orderByType, " NULLS LAST"),
        "refrigerationConsumptionByArea" => format!("{}{}{}", " ORDER BY refrigeration_consumption_by_area ", orderByType, " NULLS LAST"),
        "cityName" => format!("{}{}{}", " ORDER BY u.city_name ", orderByType, " NULLS LAST"),
        "stateName" => format!("{}{}{}", " ORDER BY u.state_name ", orderByType, " NULLS LAST"),
        "totalCharged" => format!("{}{}{}", " ORDER BY total_charged ", orderByType, " NULLS LAST"),
        "procelCategory" => format!("{}{}{}", " ORDER BY category ", orderByType, " NULLS LAST"),
        "clientName" => format!("{}{}", " ORDER BY client_name ", orderByType),
        "consumptionPreviousPercentage" => format!("{}{}{}", " ORDER BY previous_consumption ", orderByType, " NULLS LAST"),
        _ => format!("{}{}{}", " ORDER BY ac.ranking ", orderByType, " NULLS LAST")
    });

    if(params.limit.is_some() && params.offset.is_some()){
        sql = format!("{}{}", sql, " LIMIT $4 OFFSET $5;");
    }

    let procel = procel_insigths(&GetProcelInsightsRequestBody{
        startDate: params.startDate.clone(),
        endDate: params.endDate.clone(),
        previousStartDate: params.previousStartDate.clone(),
        previousEndDate: params.previousEndDate.clone(),
        units: params.units.clone(),
        minConsumption: params.minConsumption.clone(),
        procelUnitsFilter: None
    }, globs).unwrap();

    let mut unitFiltereds: Vec<i32> = Vec::new();

    if params.categoryFilter.is_some() {
        if params.categoryFilter.clone().unwrap().len() > 0 {
            for category in params.categoryFilter.clone().unwrap().iter() {
                match category {
                    crate::http::structs::energy::categoryFilterEnum::A => unitFiltereds.extend(procel.classA.units.clone()),
                    crate::http::structs::energy::categoryFilterEnum::B => unitFiltereds.extend(procel.classB.units.clone()),
                    crate::http::structs::energy::categoryFilterEnum::C => unitFiltereds.extend(procel.classC.units.clone()),
                    crate::http::structs::energy::categoryFilterEnum::D => unitFiltereds.extend(procel.classD.units.clone()),
                    crate::http::structs::energy::categoryFilterEnum::E => unitFiltereds.extend(procel.classE.units.clone()),
                    crate::http::structs::energy::categoryFilterEnum::F => unitFiltereds.extend(procel.classF.units.clone()),
                    crate::http::structs::energy::categoryFilterEnum::G => unitFiltereds.extend(procel.classG.units.clone())
                }
            }
        }
    };

    let sqlQuery = sql_query(sql)
        .bind::<Array<Integer>, _>(params.units.clone())
        .bind::<Text, _>(params.startDate.clone())
        .bind::<Text, _>(params.endDate.clone())
        .bind::<Nullable<Integer>, _>(params.limit.unwrap_or_default())
        .bind::<Nullable<Integer>, _>(params.offset.unwrap_or_default())
        .bind::<Nullable<Array<Integer>>, _>(procel.classA.units.clone())
        .bind::<Nullable<Array<Integer>>, _>(procel.classB.units.clone())
        .bind::<Nullable<Array<Integer>>, _>(procel.classC.units.clone())
        .bind::<Nullable<Array<Integer>>, _>(procel.classD.units.clone())
        .bind::<Nullable<Array<Integer>>, _>(procel.classE.units.clone())
        .bind::<Nullable<Array<Integer>>, _>(procel.classF.units.clone())
        .bind::<Nullable<Array<Integer>>, _>(procel.classG.units.clone())
        .bind::<Integer, _>(params.minConsumption.clone())
        .bind::<Text, _>(params.previousStartDate.clone())
        .bind::<Text, _>(params.previousEndDate.clone())
        .bind::<Nullable<Array<Integer>>, _>(unitFiltereds.clone());    

    let response = sqlQuery.load::<GetEnergyAnalysisListResponseSQL>(&mut conn)?;
    
    Ok(GetEnergyAnalysisListResponse{
        units: response,
        classA: procel.classA.units.len() as i32,
        classB: procel.classB.units.len() as i32,
        classC: procel.classC.units.len() as i32,
        classD: procel.classD.units.len() as i32,
        classE: procel.classE.units.len() as i32,
        classF: procel.classF.units.len() as i32,
        classG: procel.classG.units.len() as i32,
    })
}

pub fn get_energy_stats_by_units(params: &GetUnitListProcelRequestBody, conn: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Vec<GetUnitEnergyStats>, diesel::result::Error> {

    let mut unitFilter = String::new();

    if !params.units.is_empty() {
        unitFilter = " AND u.reference_id = ANY($1::integer[]) ".to_string();
    };

    let mut sql = format!("
        WITH consumption_stats AS (
            SELECT
                u.id as unit,
                ROUND((sum(ehv.consumption) / u.constructed_area) , 2) as consumption_by_area
            FROM
                energy_hist_view ehv 
                join units u on u.id = ehv.unit_id
            WHERE
                u.constructed_area is not null
                AND ehv.compilation_record_date BETWEEN to_timestamp($2, 'YYYY-MM-DD') AND to_timestamp($3, 'YYYY-MM-DD')
                {}
            GROUP BY
                u.id, u.constructed_area
            HAVING
                SUM(ehv.consumption) >= $4
        )
        SELECT
            MIN(cs.consumption_by_area) as min_consumption_by_area,
            MAX(cs.consumption_by_area) as max_consumption_by_area,
            AVG(cs.consumption_by_area) as avg_consumption_by_area
        FROM
            consumption_stats cs;", unitFilter);

    let sqlQuery = sql_query(sql)
        .bind::<Array<Integer>, _>(params.units.clone())
        .bind::<Text, _>(params.startDate.clone())
        .bind::<Text, _>(params.endDate.clone())
        .bind::<Integer, _>(params.minConsumption.clone());

   let response = sqlQuery.load::<GetUnitEnergyStats>(conn)?;

    Ok(response)
}

pub fn get_consumption_by_area_by_units(params: &GetUnitListProcelRequestBody, conn: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Vec<GetUnitConsumptionByArea>, diesel::result::Error> {

    let mut unitFilter = String::new();

    if !params.units.is_empty() {
        unitFilter = " AND u.reference_id = ANY($1::integer[]) ".to_string();
    };

    let mut sql = format!("
        SELECT
            u.reference_id as unit_id,
            ROUND((SUM(ehv.consumption) / u.constructed_area), 2) as consumption_by_area
        FROM
            energy_hist_view ehv
            JOIN
                units u ON ehv.unit_id = u.id
        WHERE
            u.constructed_area IS NOT NULL
            AND ehv.compilation_record_date BETWEEN to_timestamp($2, 'YYYY-MM-DD') AND to_timestamp($3, 'YYYY-MM-DD')
            {}
        GROUP BY
            u.reference_id, u.constructed_area
        HAVING
            SUM(ehv.consumption) >= $4 
        ORDER BY
            consumption_by_area ASC", unitFilter);


    let sqlQuery = sql_query(sql)
        .bind::<Array<Integer>, _>(params.units.clone())
        .bind::<Text, _>(params.startDate.clone())
        .bind::<Text, _>(params.endDate.clone())
        .bind::<Integer, _>(params.minConsumption.clone());

   let response = sqlQuery.load::<GetUnitConsumptionByArea>(conn)?;
    Ok(response)
}

pub fn get_consumption_stats_by_units(params: &GetUnitListProcelRequestBody, conn: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Vec<GetGeneralUnitsStats>, diesel::result::Error> {

    let mut unitFilter = String::new();

    if !params.units.is_empty() {
        unitFilter = " AND u.reference_id = ANY($1::integer[]) ".to_string();
    };
    let mut sql = format!("
        WITH sum_consumption AS (
            SELECT
                u.id as unit,
                SUM(ehv.consumption) as consumption,
                CASE
                    WHEN u.tarifa_kwh is null THEN null
                    ELSE SUM(ehv.consumption) * u.tarifa_kwh
                END AS total_charged
            FROM
                energy_hist_view ehv 
                JOIN units u on u.id = ehv.unit_id
            WHERE
                ehv.compilation_record_date BETWEEN to_timestamp($2, 'YYYY-MM-DD') AND to_timestamp($3, 'YYYY-MM-DD')
                {}
            GROUP BY
                u.id
        )
        SELECT
            SUM(sc.consumption) as total_consumption,
            SUM(sc.total_charged) as total_charged
        FROM
            sum_consumption sc;", unitFilter);

    let sqlQuery = sql_query(sql)
        .bind::<Array<Integer>, _>(params.units.clone())
        .bind::<Text, _>(params.startDate.clone())
        .bind::<Text, _>(params.endDate.clone());

   let response = sqlQuery.load::<GetGeneralUnitsStats>(conn)?;

   Ok(response)
}

pub fn procel_insigths(params: &GetProcelInsightsRequestBody, globs: &Arc<GlobalVars>) -> Result<(GetProcelInsigthsResponse), Box<dyn Error>> {

    let mut pool = globs.pool.get()?;

    let consumptionStats = get_consumption_stats_by_units(&GetUnitListProcelRequestBody {
        startDate: params.startDate.clone(),
        endDate: params.endDate.clone(),
        units: params.units.clone(),
        minConsumption: params.minConsumption.clone()
    }, &mut pool);

    let mut generalStats = match consumptionStats {
        Ok(consumptionStat) => consumptionStat[0], 
        Err(e) => return Ok(GetProcelInsigthsResponse {

            averageConsumption: Decimal::new(0, 0),
            averageConsumptionPreviousMonthPercentage: Decimal::new(0, 0),
            totalConsumption: Decimal::new(0, 0),
            totalCharged: Decimal::new(0, 0),
            containsAnalysisData: false,
            containsProcel: false,

            classA: ProcelType {
                units: Vec::new(),
                percentage: 0.0
            },
            classB: ProcelType {
                units: Vec::new(),
                percentage: 0.0
            },
            classC: ProcelType {
                units: Vec::new(),
                percentage: 0.0
            },
            classD: ProcelType {
                units: Vec::new(),
                percentage: 0.0
            },
            classE: ProcelType {
                units: Vec::new(),
                percentage: 0.0
            },
            classF: ProcelType {
                units: Vec::new(),
                percentage: 0.0
            },
            classG: ProcelType {
                units: Vec::new(),
                percentage: 0.0
            }
        })
    };

    let stats = get_energy_stats_by_units(&GetUnitListProcelRequestBody {
        startDate: params.startDate.clone(),
        endDate: params.endDate.clone(),
        units: params.units.clone(),
        minConsumption: params.minConsumption.clone(),

    }, &mut pool);

    let procelUnits = get_consumption_by_area_by_units(&GetUnitListProcelRequestBody {
        startDate: params.startDate.clone(),
        endDate: params.endDate.clone(),
        units: params.units.clone(),
        minConsumption: params.minConsumption.clone()
    }, &mut pool);

    if stats.is_err() || procelUnits.is_err() {
        return Ok(GetProcelInsigthsResponse {
    
            averageConsumption: Decimal::new(0, 0),
            averageConsumptionPreviousMonthPercentage: Decimal::new(0, 0),
            totalConsumption: generalStats.total_consumption.unwrap_or_default(),
            totalCharged: generalStats.total_charged.unwrap_or_default(),
            containsAnalysisData: true,
            containsProcel: false,
            classA: ProcelType {
                units: Vec::new(),
                percentage: 0.0
            },
            classB: ProcelType {
                units: Vec::new(),
                percentage: 0.0
            },
            classC: ProcelType {
                units: Vec::new(),
                percentage: 0.0
            },
            classD: ProcelType {
                units: Vec::new(),
                percentage: 0.0
            },
            classE: ProcelType {
                units: Vec::new(),
                percentage: 0.0
            },
            classF: ProcelType {
                units: Vec::new(),
                percentage: 0.0
            },
            classG: ProcelType {
                units: Vec::new(),
                percentage: 0.0
            }
        })
    }

    let mut energyStats = stats.unwrap()[0];
    let units = procelUnits.unwrap();

    let mut previousMonthUnit = get_energy_stats_by_units(&GetUnitListProcelRequestBody {
        startDate: params.previousStartDate.clone(),
        endDate: params.previousEndDate.clone(),
        units: params.units.clone(),
        minConsumption: params.minConsumption.clone(),
    }, &mut pool);

    let supVariation = energyStats.avg_consumption_by_area - units[0].consumption_by_area;
    let infVariation = units.last().unwrap().consumption_by_area - energyStats.avg_consumption_by_area;

    let mut classA: Vec<i32> = Vec::new();
    let mut classB: Vec<i32> = Vec::new();
    let mut classC: Vec<i32> = Vec::new();
    let mut classD: Vec<i32> = Vec::new();
    let mut classE: Vec<i32> = Vec::new();
    let mut classF: Vec<i32> = Vec::new();
    let mut classG: Vec<i32> = Vec::new();

    let classASup = (units[0].consumption_by_area).to_f64().unwrap_or_default();
    let classBSup = (units[0].consumption_by_area + (supVariation * Decimal::new(285, 3))).to_f64().unwrap_or_default();
    let classCSup = (units[0].consumption_by_area + (supVariation * Decimal::new(570, 3))).to_f64().unwrap_or_default();
    let classDSup = (units[0].consumption_by_area + (supVariation * Decimal::new(855, 3))).to_f64().unwrap_or_default();
    let classESup = (units.last().unwrap().consumption_by_area - (infVariation * Decimal::new(855, 3))).to_f64().unwrap_or_default();
    let classFSup = (units.last().unwrap().consumption_by_area - (infVariation * Decimal::new(570, 3))).to_f64().unwrap_or_default();
    let classGSup = (units.last().unwrap().consumption_by_area - (infVariation * Decimal::new(258, 3))).to_f64().unwrap_or_default();

    let classAInf = (units[0].consumption_by_area + (supVariation * Decimal::new(285, 3))).to_f64().unwrap_or_default();
    let classBInf = (units[0].consumption_by_area + (supVariation * Decimal::new(570, 3))).to_f64().unwrap_or_default();
    let classCInf = (units[0].consumption_by_area + (supVariation * Decimal::new(855, 3))).to_f64().unwrap_or_default();
    let classDInf = (units.last().unwrap().consumption_by_area - (infVariation * Decimal::new(855, 3))).to_f64().unwrap_or_default();
    let classEInf = (units.last().unwrap().consumption_by_area - (infVariation * Decimal::new(570, 3))).to_f64().unwrap_or_default();
    let classFInf = (units.last().unwrap().consumption_by_area - (infVariation * Decimal::new(258, 3))).to_f64().unwrap_or_default();
    let classGInf = (units.last().unwrap().consumption_by_area).to_f64().unwrap_or_default();

    for unit in units.iter(){
        match unit.consumption_by_area.to_f64().unwrap_or_default() {
            value if (classASup ..= classAInf).contains(&value) => classA.push(unit.unit_id),
            value if (classBSup ..= classBInf).contains(&value) => classB.push(unit.unit_id),
            value if (classCSup ..= classCInf).contains(&value) => classC.push(unit.unit_id),
            value if (classDSup ..= classDInf).contains(&value) => classD.push(unit.unit_id),
            value if (classESup ..= classEInf).contains(&value) => classE.push(unit.unit_id),
            value if (classFSup ..= classFInf).contains(&value) => classF.push(unit.unit_id),
            value if (classGSup ..= classGInf).contains(&value) => classG.push(unit.unit_id),
            _ => ()
        }
    }

    if(params.procelUnitsFilter.is_some()){

        generalStats = get_consumption_stats_by_units(&GetUnitListProcelRequestBody{
            startDate: params.startDate.clone(),
            endDate: params.endDate.clone(),
            minConsumption: params.minConsumption.clone(),
            units: params.procelUnitsFilter.clone().unwrap()
        }, &mut pool).unwrap()[0];

        energyStats = get_energy_stats_by_units(&GetUnitListProcelRequestBody {
            startDate: params.startDate.clone(),
            endDate: params.endDate.clone(),
            minConsumption: params.minConsumption.clone(),
            units: params.procelUnitsFilter.clone().unwrap()
        }, &mut pool).unwrap()[0];

        previousMonthUnit = get_energy_stats_by_units(&GetUnitListProcelRequestBody {
            startDate: params.previousStartDate.clone(),
            endDate: params.previousEndDate.clone(),
            units: params.procelUnitsFilter.clone().unwrap(),
            minConsumption: params.minConsumption.clone()
        }, &mut pool)

    }

    drop(pool);

    let response = GetProcelInsigthsResponse {

        averageConsumption: energyStats.avg_consumption_by_area.ceil(),
        averageConsumptionPreviousMonthPercentage: match previousMonthUnit {
            Ok(previousStats) => format!("{:.2}", ((energyStats.avg_consumption_by_area.ceil() - previousStats[0].avg_consumption_by_area.ceil()) / previousStats[0].avg_consumption_by_area.ceil()) * Decimal::new(100, 0)).parse().unwrap_or_default(),
            _ => Decimal::new(0, 0)
        },
        totalConsumption: generalStats.total_consumption.unwrap_or_default(),
        totalCharged: generalStats.total_charged.unwrap_or_default(),
        containsAnalysisData: true,
        containsProcel: match units.is_empty() {
            true => false,
            false => true
        },

        classA: ProcelType {
            units: classA.clone(),
            percentage: format!("{:.2}", (classA.len().to_f64().unwrap_or_default() / units.len().to_f64().unwrap_or_default()) * 100.0).parse().unwrap_or_default()
        },
        classB: ProcelType {
            units: classB.clone(),
            percentage: format!("{:.2}", (classB.len().to_f64().unwrap_or_default() / units.len().to_f64().unwrap_or_default()) * 100.0).parse().unwrap_or_default()
        },
        classC: ProcelType {
            units: classC.clone(),
            percentage: format!("{:.2}", (classC.len().to_f64().unwrap_or_default() / units.len().to_f64().unwrap_or_default()) * 100.0).parse().unwrap_or_default()
        },
        classD: ProcelType {
            units: classD.clone(),
            percentage: format!("{:.2}", (classD.len().to_f64().unwrap_or_default() / units.len().to_f64().unwrap_or_default()) * 100.0).parse().unwrap_or_default()
        },
        classE: ProcelType {
            units: classE.clone(),
            percentage: format!("{:.2}", (classE.len().to_f64().unwrap_or_default() / units.len().to_f64().unwrap_or_default()) * 100.0).parse().unwrap_or_default()
        },
        classF: ProcelType {
            units: classF.clone(),
            percentage: format!("{:.2}", (classF.len().to_f64().unwrap_or_default() / units.len().to_f64().unwrap_or_default()) * 100.0).parse().unwrap_or_default()
        },
        classG: ProcelType {
            units: classG.clone(),
            percentage: format!("{:.2}", (classG.len().to_f64().unwrap_or_default() / units.len().to_f64().unwrap_or_default()) * 100.0).parse().unwrap_or_default()
        }
    };

    Ok(response)
}

pub fn get_energy_consumption_average(params: GetEnergyAverage, globs: &Arc<GlobalVars>) -> Result<(GetEnergyAverageResponse), Box<dyn Error>> {
    let mut pool = globs.pool.get()?;

    let mut sql = "
        WITH last_consumptions AS (
            SELECT 
                eh.consumption as consumption,
                eh.record_date as date
            FROM
                energy_hist eh 
            WHERE
                eh.record_date between to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS') - interval '3 month' and to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS')
                AND EXTRACT (hour from eh.record_date) = EXTRACT (hour from to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'))
                AND EXTRACT (dow from eh.record_date) = EXTRACT (dow from to_timestamp($1, 'YYYY-MM-DD HH24:MI:SS'))
                AND electric_circuit_id = $2
                AND eh.is_valid_consumption = true 
            ORDER BY
                eh.record_date Desc
            LIMIT 4
            OFFSET 0
        )
        SELECT
            ROUND(AVG(lc.consumption), 2) as consumption_average
        FROM
            last_consumptions lc".to_string();

    let sqlQuery = sql_query(sql)
        .bind::<Text, _>(params.date.clone())
        .bind::<Integer, _>(params.electCircuitId.clone());

    let response = sqlQuery.load::<GetEnergyAverageResponse>(&mut pool)?;

    drop(pool);
        
    Ok(response[0])
}

pub fn get_total_days_unit_with_consumption(params: ParamsGetTotalDaysConsumptionUnit, globs: &Arc<GlobalVars>) -> Result<GetTotalDaysConsumptionUnit, Box<dyn Error>> {
    let mut conn = PgConnection::establish(&globs.configfile.POSTGRES_DATABASE_URL).expect("CONNECTION DB ERROR: ");

    let sql = "
        SELECT
            COALESCE(COUNT(DISTINCT time_bucket('1 day', eh.record_date)), 0) AS days_count
        FROM 
            energy_hist eh
            INNER JOIN electric_circuits ec ON ec.id = eh.electric_circuit_id 
        	INNER JOIN units u ON ec.unit_id = u.id
        WHERE
            u.id = $1 and
            eh.record_date BETWEEN to_timestamp($2, 'YYYY-MM-DD') AND to_timestamp($3, 'YYYY-MM-DD')
        ";

    let sql_query_aux = sql_query(sql)
        .bind::<Integer, _>(params.unit_id)
        .bind::<Text, _>(params.start_date.clone())
        .bind::<Text, _>(params.end_date.clone());

   let response = sql_query_aux.get_result::<GetTotalDaysConsumptionUnit>(&mut conn)?;
        
    Ok(response)
}
