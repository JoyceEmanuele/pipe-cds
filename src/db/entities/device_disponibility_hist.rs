use diesel::prelude::*;
use diesel::upsert::excluded;
use crate::models::database_models::device_disponibility_hist::DeviceDisponibilityHist;
use crate::schedules::scheduler::write_to_log_file_thread;
use crate::schema::device_disponibility_hist;
use crate::GlobalVars;
use std::sync::Arc;
use std::error::Error;

pub fn insert_data_device_disponibility_hist(data: DeviceDisponibilityHist, globs: &Arc<GlobalVars>) -> Result<(), Box<dyn Error>> {
    let mut pool = globs.pool.get()?;
    
    let result = diesel::insert_into(device_disponibility_hist::table)
        .values(&data)
        .on_conflict((device_disponibility_hist::unit_id, device_disponibility_hist::device_code, device_disponibility_hist::record_date))
        .do_update()
        .set((
            device_disponibility_hist::disponibility.eq(excluded(device_disponibility_hist::disponibility)),
        ))
        .execute(&mut pool);

    match result {
        Ok(_) => {}
        Err(err) => {
            write_to_log_file_thread(&format!("Error entering data in device_disponibility_hist, {:?}", err), 0, "ERROR");
            eprintln!("Error when entering data: {:?}, {}", data, err);
        }
    }
    
    drop(pool);
    
    Ok(())
}
