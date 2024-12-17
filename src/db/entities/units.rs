use diesel::sql_types::Integer;
use diesel::{prelude::*, sql_query};
use crate::models::database_models::units::Units;
use crate::models::external_models::unit::UnitInfo;
use crate::schedules::scheduler::write_to_log_file_thread;
use crate::{schema, GlobalVars};
use std::sync::Arc;
use std::error::Error;

pub fn insert_data_unit(data: Units, globs: &Arc<GlobalVars>) -> Result<i32, Box<dyn Error>> {
    let mut pool = globs.pool.get()?;
    let result = diesel::insert_into(schema::units::table)
        .values(&data)
        .execute(&mut pool);

    match result {
        Ok(rows_affected) => {}
        Err(err) => {
            write_to_log_file_thread(&format!("Error inserting data in units, {:?}", err), 0, "ERROR");
            eprintln!("Inserting data error: {:?}, {}", data, err);
        }
    }

    let inserted_id: i32 = schema::units::table
        .select(schema::units::id)
        .filter(schema::units::unit_name.eq(&data.unit_name))
        .first(&mut pool)?;
    
    drop(pool);

    Ok(inserted_id)
}


pub fn get_unit(reference_unit_id: i32, globs: &Arc<GlobalVars>) -> Result<Option<Units>, Box<dyn Error>> {
    let mut pool = globs.pool.get()?;

    let sql = "
        SELECT 
            id,
            client_id,
            unit_name,
            reference_id,
            city_name,
            state_name,
            tarifa_kwh,
            constructed_area,
            capacity_power
        FROM
            units
        WHERE
            reference_id = $1".to_string();

    let sqlQuery = sql_query(sql).bind::<Integer, _>(reference_unit_id);

    let result = sqlQuery.load::<Units>(&mut pool)?;

    drop(pool);

    match result.first() {
        Some(unit) => Ok(Some(Units {
            id: result[0].id,
            client_id: result[0].client_id,
            unit_name: result[0].unit_name.clone(),
            reference_id: result[0].reference_id,
            city_name: result[0].city_name.clone(),
            state_name: result[0].state_name.clone(),
            tarifa_kwh: result[0].tarifa_kwh.clone(),
            constructed_area: result[0].constructed_area.clone(),
            capacity_power: result[0].capacity_power.clone()
        })),
        None => Ok(None)
    }
}

pub fn update_unit(unit_info: &UnitInfo, globs: &Arc<GlobalVars>) -> Result<(), Box<dyn Error>> {
    let mut pool = globs.pool.get()?;

    diesel::update(schema::units::table.filter(schema::units::reference_id.eq(unit_info.unit_id)))
        .set(
            (schema::units::unit_name.eq(unit_info.unit_name.clone()),
            schema::units::city_name.eq(unit_info.city_name.clone()),
            schema::units::state_name.eq(unit_info.state_name.clone()),
            schema::units::tarifa_kwh.eq(unit_info.tarifa_kwh.clone()),
            schema::units::constructed_area.eq(unit_info.constructed_area.clone()),
            schema::units::capacity_power.eq(unit_info.capacity_power.clone())
        ))
        .execute(&mut pool)?;
    
    drop(pool);
    
    Ok(())
}
