use diesel::upsert::excluded;
use diesel::{prelude::*, sql_query};
use crate::models::database_models::devices_l1_totalization_hist::DevicesL1TotalizationHist;
use crate::schedules::scheduler::write_to_log_file_thread;
use crate::schema::devices_l1_totalization_hist;
use crate::{schema, GlobalVars};
use std::sync::Arc;
use std::error::Error;

pub fn insert_data_device_l1_totalization_hist(data: DevicesL1TotalizationHist, globs: &Arc<GlobalVars>) -> Result<(), Box<dyn Error>> {
    let mut pool = globs.pool.get()?;
    let result = diesel::insert_into(schema::devices_l1_totalization_hist::table)
        .values(&data)
        .on_conflict((devices_l1_totalization_hist::device_code, devices_l1_totalization_hist::record_date))
        .do_update()
        .set((
            devices_l1_totalization_hist::seconds_on.eq(excluded(devices_l1_totalization_hist::seconds_on)),
            devices_l1_totalization_hist::seconds_off.eq(excluded(devices_l1_totalization_hist::seconds_off)),
            devices_l1_totalization_hist::seconds_on_outside_programming.eq(excluded(devices_l1_totalization_hist::seconds_on_outside_programming)),
            devices_l1_totalization_hist::seconds_must_be_off.eq(excluded(devices_l1_totalization_hist::seconds_must_be_off)),
            devices_l1_totalization_hist::percentage_on_outside_programming.eq(excluded(devices_l1_totalization_hist::percentage_on_outside_programming)),
            devices_l1_totalization_hist::programming.eq(excluded(devices_l1_totalization_hist::programming)),
        ))
        .execute(&mut pool);

    match result {
        Ok(_) => {}
        Err(err) => {
            write_to_log_file_thread(&format!("Erro ao inserir dados na devices_l1_totalization_hist, {:?}", err), 0, "ERROR");
            eprintln!("Erro ao inserir dados: {:?}, {}", data, err);
        }
    }
    
    drop(pool);

    Ok(())
}
