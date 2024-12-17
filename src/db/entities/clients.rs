use diesel::prelude::*;
use crate::models::database_models::clients::Clients;
use crate::schedules::scheduler::write_to_log_file_thread;
use crate::{schema, GlobalVars};
use std::sync::Arc;
use std::error::Error;

pub fn insert_data_client(data: Clients, globs: &Arc<GlobalVars>) -> Result<(i32, Option<i32>), Box<dyn Error>> {
    let mut pool = globs.pool.get()?;
    
    let result = diesel::insert_into(schema::clients::table)
    .values(&data)
    .execute(&mut pool);

    match result {
        Ok(rows_affected) => {}
        Err(err) => {
            write_to_log_file_thread(&format!("Error inserting data in clients, {:?}", err), 0, "ERROR");
            eprintln!("Inserting data error: {:?}, {}", data, err);
        }
    }

    let inserted_info: (i32, Option<i32>) = schema::clients::table
    .select((schema::clients::id, schema::clients::amount_minutes_check_offline))
    .filter(schema::clients::client_name.eq(&data.client_name))
    .first(&mut pool)?;

    drop(pool);

    Ok(inserted_info)
}

pub fn get_client(client_name: &str, globs: &Arc<GlobalVars>) -> Result<Option<Clients>, Box<dyn Error>> {
    let mut pool = globs.pool.get()?;

    let client_result: Option<(i32, String, Option<i32>)> = schema::clients::table
        .filter(schema::clients::client_name.eq(client_name))
        .first(&mut pool)
        .optional()?;

    drop(pool);

    match client_result {
        Some((id, client_name, minutes_check_offline)) => {
            let client_info = Clients {
                id: Some(id),
                client_name,
                amount_minutes_check_offline: minutes_check_offline,
            };
    
            Ok(Some(client_info))
        },
        None => Ok(None)
    }
}
