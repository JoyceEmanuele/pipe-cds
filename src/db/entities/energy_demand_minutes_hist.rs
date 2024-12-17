use diesel::sql_types::{Array, Integer, Text};
use diesel::upsert::excluded;
use diesel::{prelude::*, sql_query};
use crate::http::structs::energy_demand::{GetDemandInfoResponse, GetEnergyDemandResponse};
use crate::models::database_models::energy_demand_minutes_hist::EnergyDemandMinutesHist;
use crate::schedules::scheduler::write_to_log_file_thread;
use crate::schema::energy_demand_minutes_hist;
use crate::GlobalVars;
use std::sync::Arc;
use std::error::Error;

pub fn insert_data_demand(data: EnergyDemandMinutesHist, globs: &Arc<GlobalVars>) -> Result<(), Box<dyn Error>> {
    let mut pool = globs.pool.get()?;
    
    let result = diesel::insert_into(energy_demand_minutes_hist::table)
        .values(&data)
        .on_conflict((energy_demand_minutes_hist::electric_circuit_id, energy_demand_minutes_hist::record_date))
        .do_update()
        .set((
            energy_demand_minutes_hist::average_demand.eq(excluded(energy_demand_minutes_hist::average_demand)),
            energy_demand_minutes_hist::max_demand.eq(excluded(energy_demand_minutes_hist::max_demand)),
            energy_demand_minutes_hist::min_demand.eq(excluded(energy_demand_minutes_hist::min_demand)),
        ))
        .execute(&mut pool);

    match result {
        Ok(_) => {}
        Err(err) => {
            let error_msg = format!("Erro ao inserir dados: {:?}, {}", data, err);
            write_to_log_file_thread(&error_msg, 0, "ERROR");
        }
    }
    
    drop(pool);
    
    Ok(())
}

pub fn get_demand_energy_grouped_by_hour(unit_id: i32, electric_circuit_ids: Vec<i32>, start_date: &str, end_date: &str, globs: &Arc<GlobalVars>) -> Result<Vec<GetEnergyDemandResponse>, Box<dyn Error>> {
    let mut pool: diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>> = globs.pool.get()?;
    
    let demand_hist = sql_query("
    SELECT
        SUM(energy_demand_hist_view.max_demand) as max_demand,
        SUM(energy_demand_hist_view.min_demand) as min_demand,
        ROUND(SUM(energy_demand_hist_view.average_demand), 2) AS average_demand,
        energy_demand_hist_view.compilation_record_date as record_date
    FROM
        energy_demand_hist_view
        INNER JOIN electric_circuits on (electric_circuits.id = energy_demand_hist_view.electric_circuit_id)
        INNER JOIN units on (units.id = electric_circuits.unit_id)
    WHERE
        electric_circuits.reference_id = ANY($1::integer[]) AND
        units.reference_id = $2 AND
        energy_demand_hist_view.compilation_record_date >= to_timestamp($3, 'YYYY-MM-DD HH24:MI:SS') AND
        energy_demand_hist_view.compilation_record_date <= to_timestamp($4, 'YYYY-MM-DD HH24:MI:SS')
    GROUP BY 
        energy_demand_hist_view.compilation_record_date
    ORDER BY
        energy_demand_hist_view.compilation_record_date ASC
    ");

    let demand_hist = demand_hist
        .bind::<Array<Integer>, _>(electric_circuit_ids.clone())
        .bind::<Integer, _>(unit_id)
        .bind::<Text, _>(start_date)
        .bind::<Text, _>(end_date);

    let response = demand_hist.load::<GetEnergyDemandResponse>(&mut pool)?;
    
    Ok(response)
}

pub fn get_demand_energy_grouped_by_minutes(unit_id: i32, electric_circuit_ids: Vec<i32>, start_date: &str, end_date: &str, globs: &Arc<GlobalVars>) -> Result<Vec<GetEnergyDemandResponse>, Box<dyn Error>> {
    let mut pool: diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>> = globs.pool.get()?;
    
    let demand_hist = sql_query("
    SELECT
        SUM(energy_demand_minutes_hist.max_demand) as max_demand,
        SUM(energy_demand_minutes_hist.min_demand) as min_demand,
        ROUND(SUM(energy_demand_minutes_hist.average_demand), 2) AS average_demand,
        energy_demand_minutes_hist.record_date
    FROM
        energy_demand_minutes_hist
        INNER JOIN electric_circuits on (electric_circuits.id = energy_demand_minutes_hist.electric_circuit_id)
        INNER JOIN units on (units.id = electric_circuits.unit_id)
    WHERE
        electric_circuits.reference_id = ANY($1::integer[]) AND
        units.reference_id = $2 AND
        energy_demand_minutes_hist.record_date >= to_timestamp($3, 'YYYY-MM-DD HH24:MI:SS') AND
        energy_demand_minutes_hist.record_date <= to_timestamp($4, 'YYYY-MM-DD HH24:MI:SS')
    GROUP BY 
        energy_demand_minutes_hist.record_date
    ORDER BY
        energy_demand_minutes_hist.record_date ASC
    ");

    let demand_hist = demand_hist
        .bind::<Array<Integer>, _>(electric_circuit_ids.clone())
        .bind::<Integer, _>(unit_id)
        .bind::<Text, _>(start_date)
        .bind::<Text, _>(end_date);

    let response = demand_hist.load::<GetEnergyDemandResponse>(&mut pool)?;
    
    Ok(response)
}

pub fn get_demand_info_by_hour(unit_id: i32, electric_circuit_ids: Vec<i32>, start_date: &str, end_date: &str, globs: &Arc<GlobalVars>) -> Result<Option<GetDemandInfoResponse>, Box<dyn Error>> {
    let mut pool: diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>> = globs.pool.get()?;
    let demand_hist = sql_query("
    WITH ranked_data AS (
    SELECT
        SUM(max_demand) as max_demand,
        SUM(min_demand) as min_demand,
        SUM(average_demand) as average_demand,
        compilation_record_date,
        ROW_NUMBER() OVER (ORDER BY SUM(max_demand) DESC) AS max_rank,
        ROW_NUMBER() OVER (ORDER BY SUM(min_demand) ASC) AS min_rank
    FROM
        energy_demand_hist_view
        INNER JOIN electric_circuits ON electric_circuits.id = energy_demand_hist_view.electric_circuit_id
        INNER JOIN units ON units.id = electric_circuits.unit_id
    WHERE
        electric_circuits.reference_id = ANY($1::integer[]) AND
        units.reference_id = $2 AND
        energy_demand_hist_view.compilation_record_date >= to_timestamp($3, 'YYYY-MM-DD HH24:MI:SS') AND
        energy_demand_hist_view.compilation_record_date <= to_timestamp($4, 'YYYY-MM-DD HH24:MI:SS')
    GROUP BY 
        energy_demand_hist_view.compilation_record_date
    )
    SELECT
        MAX(max_demand) AS max_demand,
        MIN(min_demand) AS min_demand,
        ROUND(AVG(average_demand), 2) AS average_demand,
        ROUND(SUM(average_demand), 2) as sum_demand,
        COUNT (*) as qtd_demand,
        MAX(CASE WHEN max_rank = 1 THEN compilation_record_date END) AS max_demand_date,
        MAX(CASE WHEN min_rank = 1 THEN compilation_record_date END) AS min_demand_date
    FROM
        ranked_data
    HAVING 
        COUNT(*) > 0;
    ");

    let demand_hist = demand_hist
        .bind::<Array<Integer>, _>(electric_circuit_ids.clone())
        .bind::<Integer, _>(unit_id)
        .bind::<Text, _>(start_date)
        .bind::<Text, _>(end_date);

    let response = demand_hist.get_result::<GetDemandInfoResponse>(&mut pool).optional()?;
    
    
    Ok(response)
}

pub fn get_demand_info_by_minutes(unit_id: i32, electric_circuit_ids: Vec<i32>, start_date: &str, end_date: &str, globs: &Arc<GlobalVars>) -> Result<Option<GetDemandInfoResponse>, Box<dyn Error>> {
    let mut pool: diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>> = globs.pool.get()?;
    
    let demand_hist = sql_query("
    WITH ranked_data AS (
    SELECT
        SUM(max_demand) as max_demand,
        SUM(min_demand) as min_demand,
        SUM(average_demand) as average_demand,
        record_date,
        ROW_NUMBER() OVER (ORDER BY SUM(max_demand) DESC) AS max_rank,
        ROW_NUMBER() OVER (ORDER BY SUM(min_demand) ASC) AS min_rank
    FROM
        energy_demand_minutes_hist
        INNER JOIN electric_circuits ON electric_circuits.id = energy_demand_minutes_hist.electric_circuit_id
        INNER JOIN units ON units.id = electric_circuits.unit_id
    WHERE
        electric_circuits.reference_id = ANY($1::integer[]) AND
        units.reference_id = $2 AND
        energy_demand_minutes_hist.record_date >= to_timestamp($3, 'YYYY-MM-DD HH24:MI:SS') AND
        energy_demand_minutes_hist.record_date <= to_timestamp($4, 'YYYY-MM-DD HH24:MI:SS')
    GROUP BY 
        energy_demand_minutes_hist.record_date
    )
    SELECT
        MAX(max_demand) AS max_demand,
        MIN(min_demand) AS min_demand,
        ROUND(AVG(average_demand), 2) AS average_demand,
        ROUND(SUM(average_demand), 2) as sum_demand,
        COUNT (*) as qtd_demand,
        MAX(CASE WHEN max_rank = 1 THEN record_date END) AS max_demand_date,
        MAX(CASE WHEN min_rank = 1 THEN record_date END) AS min_demand_date
    FROM
        ranked_data
    HAVING
        COUNT(*) > 0;
    ");

    let demand_hist = demand_hist
        .bind::<Array<Integer>, _>(electric_circuit_ids.clone())
        .bind::<Integer, _>(unit_id)
        .bind::<Text, _>(start_date)
        .bind::<Text, _>(end_date);

    let response = demand_hist.get_result::<GetDemandInfoResponse>(&mut pool).optional()?;
    
    
    Ok(response)
}
