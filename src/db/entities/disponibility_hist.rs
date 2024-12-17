use chrono::NaiveDate;
use diesel::prelude::*;
use diesel::upsert::excluded;
use crate::models::database_models::disponibility_hist::DisponibilityHist;
use crate::schedules::scheduler::write_to_log_file_thread;
use crate::schema::disponibility_hist;
use crate::GlobalVars;
use std::sync::Arc;
use std::error::Error;

pub fn insert_data_disponibility_hist(data: DisponibilityHist, globs: &Arc<GlobalVars>) -> Result<(), Box<dyn Error>> {
    let mut pool = globs.pool.get()?;
    
    let result = diesel::insert_into(disponibility_hist::table)
        .values(&data)
        .on_conflict((disponibility_hist::unit_id, disponibility_hist::record_date))
        .do_update()
        .set((
            disponibility_hist::disponibility.eq(excluded(disponibility_hist::disponibility)),
        ))
        .execute(&mut pool);

    match result {
        Ok(rows_affected) => {}
        Err(err) => {
            write_to_log_file_thread(&format!("Error inserting data in disponibility_hist, {:?}", err), 0, "ERROR");
            eprintln!("Inserting data error: {:?}, {}", data, err);
        }
    }
    
    drop(pool);
    
    Ok(())
}

pub fn delete_data_disponibility_hist(unit_id: i32, production_timestamp: &str, globs: &Arc<GlobalVars>) -> Result<(), Box<dyn Error>> {
    let mut pool = globs.pool.get()?;

    let parsed_date = NaiveDate::parse_from_str(production_timestamp, "%Y-%m-%dT%H:%M:%S%.3fZ")
        .map_err(|e| format!("Failed to parse production_timestamp: {}", e))?;

    diesel::delete(disponibility_hist::table
        .filter(disponibility_hist::unit_id.eq(unit_id))
        .filter(disponibility_hist::record_date.lt(parsed_date)))
        .execute(&mut pool)?;

    drop(pool);

    Ok(())
} 
