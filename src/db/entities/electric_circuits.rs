use diesel::prelude::*;
use crate::models::database_models::electric_circuits::ElectricCircuit;
use crate::schedules::scheduler::write_to_log_file_thread;
use crate::{schema, GlobalVars};
use std::sync::Arc;
use std::error::Error;

pub fn insert_data_electric_circuits(data: ElectricCircuit, globs: &Arc<GlobalVars>) -> Result<i32, Box<dyn Error>> {
    let mut pool = globs.pool.get()?;

    let result = diesel::insert_into(schema::electric_circuits::table)
    .values(&data)
    .returning(schema::electric_circuits::id)
    .get_result::<i32>(&mut pool);

    match result {
        Ok(inserted_id) => {
            Ok(inserted_id)
        }
        Err(err) => {
            write_to_log_file_thread(&format!("Error inserting data in electric_circuits, {:?}", err), 0, "ERROR");
            eprintln!("Inserting data error: {:?}, {}", data, err);
            Err(err.into())
        }
    }
}


pub fn get_electric_circuit(reference_electric_circuit_id: i32, globs: &Arc<GlobalVars>) -> Result<Option<(i32, String)>, Box<dyn Error>> {
    let mut pool = globs.pool.get()?;

    let electric_circuit_result: Option<(i32, String)> = schema::electric_circuits::table
        .filter(schema::electric_circuits::reference_id.eq(reference_electric_circuit_id))
        .select((schema::electric_circuits::id, schema::electric_circuits::name))
        .first(&mut pool)
        .optional()?;
    
    drop(pool);

    match electric_circuit_result {
        Some((id, name)) => {
            Ok(Some((id, name)))
        },
        None => Ok(None)
    }
}

pub fn update_electric_circuit(electric_circuit_id: i32, name: &str, globs: &Arc<GlobalVars>) -> Result<(), Box<dyn Error>> {
    let mut pool = globs.pool.get()?;

    diesel::update(schema::electric_circuits::table.filter(schema::electric_circuits::id.eq(electric_circuit_id)))
        .set(schema::electric_circuits::name.eq(name))
        .execute(&mut pool)?;
    
    drop(pool);

    Ok(())
}
