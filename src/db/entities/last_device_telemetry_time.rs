use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::upsert::excluded;
use crate::models::database_models::last_device_telemetry_time::{GetLastTelemetryTime, LastDeviceTelemetryTime};
use crate::schedules::scheduler::write_to_log_file_thread;
use crate::schema::{self, last_device_telemetry_time};
use crate::GlobalVars;
use std::sync::Arc;
use std::error::Error;

pub fn insert_last_device_telemetry_time (data: LastDeviceTelemetryTime, globs: &Arc<GlobalVars>) -> Result<(), Box<dyn Error>> {
    let mut pool = globs.pool.get()?;
    
    let result = diesel::insert_into(last_device_telemetry_time::table)
        .values(&data)
        .on_conflict((last_device_telemetry_time::device_code))
        .do_update()
        .set((
            last_device_telemetry_time::record_date.eq(excluded(last_device_telemetry_time::record_date)),
        ))
        .execute(&mut pool);

    match result {
        Ok(_) => {}
        Err(err) => {
            write_to_log_file_thread(&format!("Error entering data in last_device_telemetry_time, {:?}", err), 0, "ERROR");
            eprintln!("Error when entering data: {:?}, {}", data, err);
        }
    }
    
    drop(pool);
    
    Ok(())
}

pub fn get_last_telemetry_time(device_code: String, globs: &Arc<GlobalVars>) -> Result<Option<GetLastTelemetryTime>, Box<dyn Error>> {
    let mut pool = globs.pool.get()?;

    let result: Option<NaiveDateTime> = schema::last_device_telemetry_time::table
        .filter(schema::last_device_telemetry_time::device_code.eq(device_code))
        .select(schema::last_device_telemetry_time::record_date)
        .first(&mut pool)
        .optional()?;

        match result {
            Some((record_date)) => {
                Ok(Some(GetLastTelemetryTime {
                    record_date,
                }))
            },
            None => Ok(None),
        }
}
