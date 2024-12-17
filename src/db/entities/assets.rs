use diesel::prelude::*;
use crate::models::database_models::assets::Assets;
use crate::{schema, GlobalVars};
use std::sync::Arc;
use std::error::Error;

pub fn insert_data_asset(data: Assets, globs: &Arc<GlobalVars>) -> Result<i32, Box<dyn Error>> {
    let mut pool = globs.pool.get()?;
    
    let result = diesel::insert_into(schema::assets::table)
        .values(&data)
        .returning(schema::assets::id)
        .get_result::<i32>(&mut pool);
    
    match result {
        Ok(inserted_id) => {
            Ok(inserted_id)
        }
        Err(err) => {
            // eprintln!("Erro ao inserir dados: {:?}, {}", data, err);
            Err(err.into())
        }
    }
}


pub fn get_asset(reference_asset_id: i32, globs: &Arc<GlobalVars>) -> Result<Option<(i32, String, String, i32)>, Box<dyn Error>> {
    let mut pool = globs.pool.get()?;

    let asset_result: Option<(i32, String, String, i32)> = schema::assets::table
        .filter(schema::assets::reference_id.eq(reference_asset_id))
        .select((schema::assets::id, schema::assets::asset_name, schema::assets::device_code, schema::assets::machine_reference_id))
        .first(&mut pool)
        .optional()?;
    
    drop(pool);

    match asset_result {
        Some((asset_id, asset_name, device_code, machine_reference_id)) => {
            Ok(Some((asset_id, asset_name, device_code, machine_reference_id)))
        },
        None => Ok(None)
    }
}

pub fn update_asset(asset_id: i32, asset_name: &str, device_code: &str, machine_reference_id: i32, globs: &Arc<GlobalVars>) -> Result<(), Box<dyn Error>> {
    let mut pool = globs.pool.get()?;

    diesel::update(schema::assets::table.filter(schema::assets::id.eq(asset_id)))
        .set((
            schema::assets::asset_name.eq(asset_name),
            schema::assets::device_code.eq(device_code),
            schema::assets::machine_reference_id.eq(machine_reference_id),
        ))
        .execute(&mut pool)?;
    
    drop(pool);
    
    Ok(())
}
