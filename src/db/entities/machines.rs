use diesel::prelude::*;
use crate::models::database_models::machines::Machines;
use crate::schedules::scheduler::write_to_log_file_thread;
use crate::{schema, GlobalVars};
use std::sync::Arc;
use std::error::Error;

pub fn insert_data_machine(data: Machines, globs: &Arc<GlobalVars>) -> Result<i32, Box<dyn Error>> {
    let mut pool = globs.pool.get()?;
    
    let result = diesel::insert_into(schema::machines::table)
        .values(&data)
        .returning(schema::machines::id)
        .get_result::<i32>(&mut pool);
    
    match result {
        Ok(inserted_id) => {
            Ok(inserted_id)
        }
        Err(err) => {
            write_to_log_file_thread(&format!("Error inserting data in machines, {:?}", err), 0, "ERROR");
            eprintln!("Inserting data error: {:?}, {}", data, err);
            Err(err.into())
        }
    }
}


pub fn get_machine(reference_machine_id: i32, globs: &Arc<GlobalVars>) -> Result<Option<(i32, String, Option<String>)>, Box<dyn Error>> {
    let mut pool = globs.pool.get()?;

    let machine_result: Option<(i32, String, Option<String>)> = schema::machines::table
        .filter(schema::machines::reference_id.eq(reference_machine_id))
        .select((schema::machines::id, schema::machines::machine_name, schema::machines::device_code_autom))
        .first(&mut pool)
        .optional()?;
    
    drop(pool);

    match machine_result {
        Some((machine_id, machine_name, device_code_autom)) => {
            Ok(Some((machine_id, machine_name, device_code_autom)))
        },
        None => Ok(None)
    }
}

pub fn update_machine(machine_id: i32, machine_name: &str, device_code_autom: &str, globs: &Arc<GlobalVars>) -> Result<(), Box<dyn Error>> {
    let mut pool = globs.pool.get()?;

    diesel::update(schema::machines::table.filter(schema::machines::id.eq(machine_id)))
        .set((
            schema::machines::machine_name.eq(machine_name),
            schema::machines::device_code_autom.eq(device_code_autom),
        ))
        .execute(&mut pool)?;

    drop(pool);

    Ok(())
}
