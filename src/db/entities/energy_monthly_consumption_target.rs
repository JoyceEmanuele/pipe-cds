use diesel::sql_types::Integer;
use diesel::{prelude::*, sql_query};
use crate::http::structs::energy::GetTotalMonthlyTarget;
use crate::models::database_models::energy_monthly_consumption_target::EnergyMonthlyConsumptionTarget;
use crate::schedules::scheduler::write_to_log_file_thread;
use crate::schema::energy_monthly_consumption_target;
use crate::GlobalVars;
use std::sync::Arc;
use std::error::Error;

pub fn insert_data_energy_monthly_consumption_target(data: EnergyMonthlyConsumptionTarget, globs: &Arc<GlobalVars>) -> Result<(), Box<dyn Error>> {
    let mut pool = globs.pool.get()?;
    
    let result = diesel::insert_into(energy_monthly_consumption_target::table)
        .values(&data)
        .on_conflict((energy_monthly_consumption_target::unit_id, energy_monthly_consumption_target::date_forecast))
        .do_nothing()
        .execute(&mut pool);

    match result {
        Ok(rows_affected) => {}
        Err(err) => {
            write_to_log_file_thread(&format!("Error inserting data in energy_monthly_consumption_target, {:?}", err), 0, "ERROR");
            eprintln!("Inserting data error: {:?}, {}", data, err);
        }
    }
    
    drop(pool);
    
    Ok(())
}

pub fn monthly_target_exists_for_unit(unit_id: i32, globs: &Arc<GlobalVars>) -> Result<GetTotalMonthlyTarget, Box<dyn Error>> {
    let mut pool = globs.pool.get().unwrap();

    let sql = "
        SELECT 
            COALESCE(COUNT(*), 0) AS monthly_target_count
        FROM 
            energy_monthly_consumption_target
        WHERE 
            unit_id = $1 
        ";
    
    let sql_query_aux = sql_query(sql)
        .bind::<Integer, _>(unit_id);

    let response = sql_query_aux.get_result::<GetTotalMonthlyTarget>(&mut pool)?;
        
    Ok(response)
}