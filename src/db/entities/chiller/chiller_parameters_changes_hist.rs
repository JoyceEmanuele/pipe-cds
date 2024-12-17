use diesel::sql_types::Text;
use diesel::{prelude::*, sql_query};
use crate::models::database_models::chiller::chiller_parameters_changes_hist::ChillerParametersChangesHist;
use crate::schedules::scheduler::write_to_log_file_thread;
use crate::schema::chiller_parameters_changes_hist;
use crate::GlobalVars;
use std::sync::Arc;
use std::error::Error;
use diesel::upsert::excluded;

pub fn insert_data_change_parameters_hist(data: ChillerParametersChangesHist, globs: &Arc<GlobalVars>) -> Result<(), Box<dyn Error>> {
    let mut pool = globs.pool.get()?;
    let result = diesel::insert_into(chiller_parameters_changes_hist::table)
        .values(&data)
        .on_conflict((chiller_parameters_changes_hist::device_code, chiller_parameters_changes_hist::record_date, chiller_parameters_changes_hist::parameter_name))
        .do_update()
        .set((
            chiller_parameters_changes_hist::parameter_value.eq(excluded(chiller_parameters_changes_hist::parameter_value)),
        ))
        .execute(&mut pool);

    match result {
        Ok(rows_affected) => {}
        Err(err) => {
            write_to_log_file_thread(&format!("Error inserting data in chiller_parameters_changes_hist, {:?}", err), 0, "ERROR");
            eprintln!("Inserting data error: {:?}, {}", data, err);
        }
    }
    
    drop(pool);

    Ok(())
}

pub fn get_chiller_parameters_changes_hist(device_code: &str, start_date: &str, end_date: &str, globs: &Arc<GlobalVars>) -> Result<Vec<ChillerParametersChangesHist>, Box<dyn Error>> {
    let mut pool = globs.pool.get()?;
    

    let params_hist = sql_query("SELECT 
        unit_id,
        record_date,
        parameter_name,
        parameter_value,
        device_code
    FROM
        chiller_parameters_changes_hist
    WHERE
        device_code = $1 AND
        record_date >= to_timestamp($2, 'YYYY-MM-DD HH24:MI:SS') AND
        record_date <= to_timestamp($3, 'YYYY-MM-DD HH24:MI:SS')
    ORDER BY record_date ASC
    ");

    let params_hist = params_hist
        .bind::<Text, _>(device_code)
        .bind::<Text, _>(start_date)
        .bind::<Text, _>(end_date);

    let response = params_hist.load::<ChillerParametersChangesHist>(&mut pool)?;
    
    Ok(response)
}
