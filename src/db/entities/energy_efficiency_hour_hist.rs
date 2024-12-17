use diesel::prelude::*;
use diesel::sql_types::{Integer, Text};
use diesel::upsert::excluded;
use diesel::sql_query;
use crate::http::structs::energy_efficiency::{GetTotalConsumptionByDeviceMachineUnitResponse, GetTotalConsumptionByUnitResponse};
use crate::models::database_models::energy_efficiency_hour_hist::EnergyEfficiencyHourHist;
use crate::schedules::scheduler::write_to_log_file_thread;
use crate::schema::energy_efficiency_hour_hist;
use crate::GlobalVars;
use std::sync::Arc;
use std::error::Error;

pub fn insert_data_energy_efficiency_hour(data: EnergyEfficiencyHourHist, globs: &Arc<GlobalVars>) -> Result<(), Box<dyn Error>> {
    let mut pool = globs.pool.get()?;
    
    let result = diesel::insert_into(energy_efficiency_hour_hist::table)
        .values(&data)
        .on_conflict((energy_efficiency_hour_hist::device_code, energy_efficiency_hour_hist::record_date))
        .do_update()
        .set((
            energy_efficiency_hour_hist::machine_id.eq(excluded(energy_efficiency_hour_hist::machine_id)),
            energy_efficiency_hour_hist::consumption.eq(excluded(energy_efficiency_hour_hist::consumption)),
            energy_efficiency_hour_hist::utilization_time.eq(excluded(energy_efficiency_hour_hist::utilization_time)),
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

pub fn get_consumption_by_unit(unit_id: i32, start_date: &str, end_date: &str, globs: &Arc<GlobalVars>) -> Result<Option<GetTotalConsumptionByUnitResponse>, Box<dyn Error>> {
    let mut pool = globs.pool.get()?;
    
    let energy_efficiency_hist = sql_query("
    SELECT 
        SUM(energy_efficiency_view_day_unit.consumption) AS total_refrigeration_consumption
    FROM
        energy_efficiency_view_day_unit
        INNER JOIN units on (units.id = energy_efficiency_view_day_unit.unit_id)
    WHERE
        units.reference_id = $1 AND
        energy_efficiency_view_day_unit.compilation_record_date >= to_timestamp($2, 'YYYY-MM-DD HH24:MI:SS') AND
        energy_efficiency_view_day_unit.compilation_record_date <= to_timestamp($3, 'YYYY-MM-DD HH24:MI:SS')
    GROUP BY units.reference_id;
    ");

    let energy_efficiency_hist = energy_efficiency_hist
        .bind::<Integer, _>(unit_id)
        .bind::<Text, _>(start_date)
        .bind::<Text, _>(end_date);

        let response = energy_efficiency_hist.get_result::<GetTotalConsumptionByUnitResponse>(&mut pool).optional()?;
    
    Ok(response)
}

pub fn get_consumption_by_device_machine_unit(unit_id: i32, start_date: &str, end_date: &str, globs: &Arc<GlobalVars>) -> Result<Vec<GetTotalConsumptionByDeviceMachineUnitResponse>, Box<dyn Error>> {
    let mut pool = globs.pool.get()?;
    
    let energy_efficiency_hist = sql_query("
    SELECT 
        machines.reference_id as machine_id,
        energy_efficiency_view_day.device_code,
        SUM(energy_efficiency_view_day.consumption) AS total_refrigeration_consumption,
        SUM(energy_efficiency_view_day.utilization_time) AS total_utilization_time
    FROM
        energy_efficiency_view_day
        INNER JOIN machines on (machines.id = energy_efficiency_view_day.machine_id)
        INNER JOIN units on (units.id = machines.unit_id)
    WHERE
        units.reference_id = $1 AND
        energy_efficiency_view_day.compilation_record_date >= to_timestamp($2, 'YYYY-MM-DD HH24:MI:SS') AND
        energy_efficiency_view_day.compilation_record_date <= to_timestamp($3, 'YYYY-MM-DD HH24:MI:SS')
    GROUP BY units.reference_id, machines.reference_id, machines.machine_name, energy_efficiency_view_day.device_code;
    ");

    let energy_efficiency_hist = energy_efficiency_hist
        .bind::<Integer, _>(unit_id)
        .bind::<Text, _>(start_date)
        .bind::<Text, _>(end_date);

    let response = energy_efficiency_hist.load::<GetTotalConsumptionByDeviceMachineUnitResponse>(&mut pool)?;
    
    Ok(response)
}
