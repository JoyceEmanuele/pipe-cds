use diesel::{prelude::*, sql_query};
use diesel::sql_types::Text;
use crate::models::database_models::chiller::chiller_xa_parameters_minute_hist::{ChillerXAParametersHistRow, ChillerXAParametersMinutesHist};
use crate::schedules::scheduler::write_to_log_file_thread;
use crate::schema::chiller_xa_parameters_minutes_hist;
use crate::GlobalVars;
use std::sync::Arc;
use std::error::Error;
use diesel::upsert::excluded;

pub fn insert_chiller_xa_parameters_hist(data: ChillerXAParametersMinutesHist, globs: &Arc<GlobalVars>) -> Result<(), Box<dyn Error>> {
    let mut pool = globs.pool.get()?;
    
    let result = diesel::insert_into(chiller_xa_parameters_minutes_hist::table)
        .values(&data)
        .on_conflict((chiller_xa_parameters_minutes_hist::device_code, chiller_xa_parameters_minutes_hist::record_date))
        .do_update()
        .set((
            chiller_xa_parameters_minutes_hist::cap_t.eq(excluded(chiller_xa_parameters_minutes_hist::cap_t)),
            chiller_xa_parameters_minutes_hist::cond_ewt.eq(excluded(chiller_xa_parameters_minutes_hist::cond_ewt)),
            chiller_xa_parameters_minutes_hist::cond_lwt.eq(excluded(chiller_xa_parameters_minutes_hist::cond_lwt)),
            chiller_xa_parameters_minutes_hist::cool_ewt.eq(excluded(chiller_xa_parameters_minutes_hist::cool_ewt)),
            chiller_xa_parameters_minutes_hist::cool_lwt.eq(excluded(chiller_xa_parameters_minutes_hist::cool_lwt)),
            chiller_xa_parameters_minutes_hist::ctrl_pnt.eq(excluded(chiller_xa_parameters_minutes_hist::ctrl_pnt)),
            chiller_xa_parameters_minutes_hist::dp_a.eq(excluded(chiller_xa_parameters_minutes_hist::dp_a)),
            chiller_xa_parameters_minutes_hist::dp_b.eq(excluded(chiller_xa_parameters_minutes_hist::dp_b)),
            chiller_xa_parameters_minutes_hist::hr_cp_a.eq(excluded(chiller_xa_parameters_minutes_hist::hr_cp_a)),
            chiller_xa_parameters_minutes_hist::hr_cp_b.eq(excluded(chiller_xa_parameters_minutes_hist::hr_cp_b)),
            chiller_xa_parameters_minutes_hist::hr_mach.eq(excluded(chiller_xa_parameters_minutes_hist::hr_mach)),
            chiller_xa_parameters_minutes_hist::hr_mach_b.eq(excluded(chiller_xa_parameters_minutes_hist::hr_mach_b)),
            chiller_xa_parameters_minutes_hist::oat.eq(excluded(chiller_xa_parameters_minutes_hist::oat)),
            chiller_xa_parameters_minutes_hist::op_a.eq(excluded(chiller_xa_parameters_minutes_hist::op_a)),
            chiller_xa_parameters_minutes_hist::op_b.eq(excluded(chiller_xa_parameters_minutes_hist::op_b)),
            chiller_xa_parameters_minutes_hist::sct_a.eq(excluded(chiller_xa_parameters_minutes_hist::sct_a)),
            chiller_xa_parameters_minutes_hist::sct_b.eq(excluded(chiller_xa_parameters_minutes_hist::sct_b)),
            chiller_xa_parameters_minutes_hist::slt_a.eq(excluded(chiller_xa_parameters_minutes_hist::slt_a)),
            chiller_xa_parameters_minutes_hist::slt_b.eq(excluded(chiller_xa_parameters_minutes_hist::slt_b)),
            chiller_xa_parameters_minutes_hist::sp.eq(excluded(chiller_xa_parameters_minutes_hist::sp)),
            chiller_xa_parameters_minutes_hist::sp_a.eq(excluded(chiller_xa_parameters_minutes_hist::sp_a)),
            chiller_xa_parameters_minutes_hist::sp_b.eq(excluded(chiller_xa_parameters_minutes_hist::sp_b)),
            chiller_xa_parameters_minutes_hist::sst_a.eq(excluded(chiller_xa_parameters_minutes_hist::sst_a)),
            chiller_xa_parameters_minutes_hist::sst_b.eq(excluded(chiller_xa_parameters_minutes_hist::sst_b)),
        ))
        .execute(&mut pool);

    match result {
        Ok(rows_affected) => {}
        Err(err) => {
            write_to_log_file_thread(&format!("Error inserting data in chiller_xa_parameters_minutes_hist, {:?}", err), 0, "ERROR");
            eprintln!("Inserting data error: {:?}, {}", data, err);
        }
    }
    
    drop(pool);
    
    Ok(())
}

pub fn get_chiller_xa_parameters_hist_minutes(device_code: &str, start_date: &str, end_date: &str, globs: &Arc<GlobalVars>) -> Result<Vec<ChillerXAParametersHistRow>, Box<dyn Error>> {
    let mut pool = globs.pool.get()?;
    
    let params_hist = sql_query("SELECT 
        unit_id,
        device_code,
        cap_t,
        cond_ewt,
        cond_lwt,
        cool_ewt,
        cool_lwt,
        ctrl_pnt,
        dp_a,
        dp_b,
        hr_cp_a,
        hr_cp_b,
        hr_mach,
        hr_mach_b,
        oat,
        op_a,
        op_b,
        sct_a,
        sct_b,
        slt_a,
        slt_b,
        sp,
        sp_a,
        sp_b,
        sst_a,
        sst_b,
        record_date
    FROM
        chiller_xa_parameters_minutes_hist
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

    let response = params_hist.load::<ChillerXAParametersHistRow>(&mut pool)?;

    Ok(response)
}

pub fn get_chiller_xa_parameters_hist_hour(device_code: &str, start_date: &str, end_date: &str, globs: &Arc<GlobalVars>) -> Result<Vec<ChillerXAParametersHistRow>, Box<dyn Error>> {
    let mut pool = globs.pool.get()?;
    
    let params_hist = sql_query("SELECT 
        unit_id,
        device_code,
        cap_t,
        cond_ewt,
        cond_lwt,
        cool_ewt,
        cool_lwt,
        ctrl_pnt,
        dp_a,
        dp_b,
        hr_cp_a,
        hr_cp_b,
        hr_mach,
        hr_mach_b,
        oat,
        op_a,
        op_b,
        sct_a,
        sct_b,
        slt_a,
        slt_b,
        sp,
        sp_a,
        sp_b,
        sst_a,
        sst_b,
        compilation_record_date as record_date
    FROM
        chiller_xa_parameters_hist_view
    WHERE
        device_code = $1 AND
        compilation_record_date >= to_timestamp($2, 'YYYY-MM-DD HH24:MI:SS') AND
        compilation_record_date <= to_timestamp($3, 'YYYY-MM-DD HH24:MI:SS')
    ORDER BY compilation_record_date ASC
    ");

    let params_hist = params_hist
        .bind::<Text, _>(device_code)
        .bind::<Text, _>(start_date)
        .bind::<Text, _>(end_date);

    let response = params_hist.load::<ChillerXAParametersHistRow>(&mut pool)?;

    Ok(response)
}
