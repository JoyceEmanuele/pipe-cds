use diesel::{prelude::*, sql_query};
use diesel::sql_types::Text;
use crate::models::database_models::chiller::chiller_hx_parameters_minute_hist::{ChillerHXParametersHistRow, ChillerHXParametersMinutesHist};
use crate::schedules::scheduler::write_to_log_file_thread;
use crate::schema::chiller_hx_parameters_minutes_hist;
use crate::GlobalVars;
use std::sync::Arc;
use std::error::Error;
use diesel::upsert::excluded;

pub fn insert_chiller_hx_parameters_hist(data: ChillerHXParametersMinutesHist, globs: &Arc<GlobalVars>) -> Result<(), Box<dyn Error>> {
    let mut pool = globs.pool.get()?;
    
    let result = diesel::insert_into(chiller_hx_parameters_minutes_hist::table)
        .values(&data)
        .on_conflict((chiller_hx_parameters_minutes_hist::device_code, chiller_hx_parameters_minutes_hist::record_date))
        .do_update()
        .set((
            chiller_hx_parameters_minutes_hist::capa_t.eq(excluded(chiller_hx_parameters_minutes_hist::capa_t)),
            chiller_hx_parameters_minutes_hist::capb_t.eq(excluded(chiller_hx_parameters_minutes_hist::capb_t)),
            chiller_hx_parameters_minutes_hist::cap_t.eq(excluded(chiller_hx_parameters_minutes_hist::cap_t)),
            chiller_hx_parameters_minutes_hist::cpa1_cur.eq(excluded(chiller_hx_parameters_minutes_hist::cpa1_cur)),
            chiller_hx_parameters_minutes_hist::cpa1_dgt.eq(excluded(chiller_hx_parameters_minutes_hist::cpa1_dgt)),
            chiller_hx_parameters_minutes_hist::cpa1_op.eq(excluded(chiller_hx_parameters_minutes_hist::cpa1_op)),
            chiller_hx_parameters_minutes_hist::cpa1_tmp.eq(excluded(chiller_hx_parameters_minutes_hist::cpa1_tmp)),
            chiller_hx_parameters_minutes_hist::cpa2_cur.eq(excluded(chiller_hx_parameters_minutes_hist::cpa2_cur)),
            chiller_hx_parameters_minutes_hist::cpa2_dgt.eq(excluded(chiller_hx_parameters_minutes_hist::cpa2_dgt)),
            chiller_hx_parameters_minutes_hist::cpa2_op.eq(excluded(chiller_hx_parameters_minutes_hist::cpa2_op)),
            chiller_hx_parameters_minutes_hist::cpa2_tmp.eq(excluded(chiller_hx_parameters_minutes_hist::cpa2_tmp)),
            chiller_hx_parameters_minutes_hist::cpb1_cur.eq(excluded(chiller_hx_parameters_minutes_hist::cpb1_cur)),
            chiller_hx_parameters_minutes_hist::cpb1_dgt.eq(excluded(chiller_hx_parameters_minutes_hist::cpb1_dgt)),
            chiller_hx_parameters_minutes_hist::cpb1_op.eq(excluded(chiller_hx_parameters_minutes_hist::cpb1_op)),
            chiller_hx_parameters_minutes_hist::cpb1_tmp.eq(excluded(chiller_hx_parameters_minutes_hist::cpb1_tmp)),
            chiller_hx_parameters_minutes_hist::cpb2_cur.eq(excluded(chiller_hx_parameters_minutes_hist::cpb2_cur)),
            chiller_hx_parameters_minutes_hist::cpb2_dgt.eq(excluded(chiller_hx_parameters_minutes_hist::cpb2_dgt)),
            chiller_hx_parameters_minutes_hist::cpb2_op.eq(excluded(chiller_hx_parameters_minutes_hist::cpb2_op)),
            chiller_hx_parameters_minutes_hist::cpb2_tmp.eq(excluded(chiller_hx_parameters_minutes_hist::cpb2_tmp)),
            chiller_hx_parameters_minutes_hist::cond_ewt.eq(excluded(chiller_hx_parameters_minutes_hist::cond_ewt)),
            chiller_hx_parameters_minutes_hist::cond_lwt.eq(excluded(chiller_hx_parameters_minutes_hist::cond_lwt)),
            chiller_hx_parameters_minutes_hist::cond_sp.eq(excluded(chiller_hx_parameters_minutes_hist::cond_sp)),
            chiller_hx_parameters_minutes_hist::cool_ewt.eq(excluded(chiller_hx_parameters_minutes_hist::cool_ewt)),
            chiller_hx_parameters_minutes_hist::cool_lwt.eq(excluded(chiller_hx_parameters_minutes_hist::cool_lwt)),
            chiller_hx_parameters_minutes_hist::ctrl_pnt.eq(excluded(chiller_hx_parameters_minutes_hist::ctrl_pnt)),
            chiller_hx_parameters_minutes_hist::dem_lim.eq(excluded(chiller_hx_parameters_minutes_hist::dem_lim)),
            chiller_hx_parameters_minutes_hist::dp_a.eq(excluded(chiller_hx_parameters_minutes_hist::dp_a)),
            chiller_hx_parameters_minutes_hist::dp_b.eq(excluded(chiller_hx_parameters_minutes_hist::dp_b)),
            chiller_hx_parameters_minutes_hist::dop_a1.eq(excluded(chiller_hx_parameters_minutes_hist::dop_a1)),
            chiller_hx_parameters_minutes_hist::dop_a2.eq(excluded(chiller_hx_parameters_minutes_hist::dop_a2)),
            chiller_hx_parameters_minutes_hist::dop_b1.eq(excluded(chiller_hx_parameters_minutes_hist::dop_b1)),
            chiller_hx_parameters_minutes_hist::dop_b2.eq(excluded(chiller_hx_parameters_minutes_hist::dop_b2)),
            chiller_hx_parameters_minutes_hist::exv_a.eq(excluded(chiller_hx_parameters_minutes_hist::exv_a)),
            chiller_hx_parameters_minutes_hist::exv_b.eq(excluded(chiller_hx_parameters_minutes_hist::exv_b)),
            chiller_hx_parameters_minutes_hist::hr_cp_a1.eq(excluded(chiller_hx_parameters_minutes_hist::hr_cp_a1)),
            chiller_hx_parameters_minutes_hist::hr_cp_a2.eq(excluded(chiller_hx_parameters_minutes_hist::hr_cp_a2)),
            chiller_hx_parameters_minutes_hist::hr_cp_b1.eq(excluded(chiller_hx_parameters_minutes_hist::hr_cp_b1)),
            chiller_hx_parameters_minutes_hist::hr_cp_b2.eq(excluded(chiller_hx_parameters_minutes_hist::hr_cp_b2)),
            chiller_hx_parameters_minutes_hist::lag_lim.eq(excluded(chiller_hx_parameters_minutes_hist::lag_lim)),
            chiller_hx_parameters_minutes_hist::sct_a.eq(excluded(chiller_hx_parameters_minutes_hist::sct_a)),
            chiller_hx_parameters_minutes_hist::sct_b.eq(excluded(chiller_hx_parameters_minutes_hist::sct_b)),
            chiller_hx_parameters_minutes_hist::sp.eq(excluded(chiller_hx_parameters_minutes_hist::sp)),
            chiller_hx_parameters_minutes_hist::sp_a.eq(excluded(chiller_hx_parameters_minutes_hist::sp_a)),
            chiller_hx_parameters_minutes_hist::sp_b.eq(excluded(chiller_hx_parameters_minutes_hist::sp_b)),
            chiller_hx_parameters_minutes_hist::sst_a.eq(excluded(chiller_hx_parameters_minutes_hist::sst_a)),
            chiller_hx_parameters_minutes_hist::sst_b.eq(excluded(chiller_hx_parameters_minutes_hist::sst_b)),
        ))
        .execute(&mut pool);

    match result {
        Ok(rows_affected) => {}
        Err(err) => {
            write_to_log_file_thread(&format!("Error inserting data in chiller_hx_parameters_minutes_hist, {:?}", err), 0, "ERROR");
            eprintln!("Inserting data error: {:?}, {}", data, err);
        }
    }
    
    drop(pool);
    
    Ok(())
}

pub fn get_chiller_hx_parameters_hist_minutes(device_code: &str, start_date: &str, end_date: &str, globs: &Arc<GlobalVars>) -> Result<Vec<ChillerHXParametersHistRow>, Box<dyn Error>> {
    let mut pool = globs.pool.get()?;
    
    let params_hist = sql_query("SELECT 
        device_code,
        unit_id,
        record_date,
        cap_t,
        dem_lim,
        lag_lim,
        sp,
        ctrl_pnt,
        capa_t,
        dp_a,
        sp_a,
        sct_a,
        sst_a,
        capb_t,
        dp_b,
        sp_b,
        sct_b,
        sst_b,
        cond_lwt,
        cond_ewt,
        cool_lwt,
        cool_ewt,
        cpa1_op,
        cpa2_op,
        dop_a1,
        dop_a2,
        cpa1_dgt,
        cpa2_dgt,
        exv_a,
        hr_cp_a1,
        hr_cp_a2,
        cpa1_tmp,
        cpa2_tmp,
        cpa1_cur,
        cpa2_cur,
        cpb1_op,
        cpb2_op,
        dop_b1,
        dop_b2,
        cpb1_dgt,
        cpb2_dgt,
        exv_b,
        hr_cp_b1,
        hr_cp_b2,
        cpb1_tmp,
        cpb2_tmp,
        cpb1_cur,
        cpb2_cur,
        cond_sp
    FROM
        chiller_hx_parameters_minutes_hist
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

    let response = params_hist.load::<ChillerHXParametersHistRow>(&mut pool)?;

    Ok(response)
}

pub fn get_chiller_hx_parameters_hist_hour(device_code: &str, start_date: &str, end_date: &str, globs: &Arc<GlobalVars>) -> Result<Vec<ChillerHXParametersHistRow>, Box<dyn Error>> {
    let mut pool = globs.pool.get()?;
    
    let params_hist = sql_query("SELECT 
        device_code,
        unit_id,
        compilation_record_date as record_date,
        cap_t,
        dem_lim,
        lag_lim,
        sp,
        ctrl_pnt,
        capa_t,
        dp_a,
        sp_a,
        sct_a,
        sst_a,
        capb_t,
        dp_b,
        sp_b,
        sct_b,
        sst_b,
        cond_lwt,
        cond_ewt,
        cool_lwt,
        cool_ewt,
        cpa1_op,
        cpa2_op,
        dop_a1,
        dop_a2,
        cpa1_dgt,
        cpa2_dgt,
        exv_a,
        hr_cp_a1,
        hr_cp_a2,
        cpa1_tmp,
        cpa2_tmp,
        cpa1_cur,
        cpa2_cur,
        cpb1_op,
        cpb2_op,
        dop_b1,
        dop_b2,
        cpb1_dgt,
        cpb2_dgt,
        exv_b,
        hr_cp_b1,
        hr_cp_b2,
        cpb1_tmp,
        cpb2_tmp,
        cpb1_cur,
        cpb2_cur,
        cond_sp
    FROM
        chiller_parameters_hist_view
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

    let response = params_hist.load::<ChillerHXParametersHistRow>(&mut pool)?;

    Ok(response)
}
