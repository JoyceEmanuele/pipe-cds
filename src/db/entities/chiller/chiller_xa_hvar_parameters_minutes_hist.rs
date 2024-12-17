use diesel::{prelude::*, sql_query};
use diesel::sql_types::Text;
use crate::models::database_models::chiller::chiller_xa_hvar_parameters_minutes_hist::{ChillerXAHvarParametersHistRow, ChillerXAHvarParametersMinutesHist};
use crate::schedules::scheduler::write_to_log_file_thread;
use crate::schema::chiller_xa_hvar_parameters_minutes_hist;
use crate::GlobalVars;
use std::sync::Arc;
use std::error::Error;
use diesel::upsert::excluded;

pub fn insert_chiller_xa_hvar_parameters_hist(data: ChillerXAHvarParametersMinutesHist, globs: &Arc<GlobalVars>) -> Result<(), Box<dyn Error>> {
    let mut pool = globs.pool.get()?;
    
    let result = diesel::insert_into(chiller_xa_hvar_parameters_minutes_hist::table)
        .values(&data)
        .on_conflict((chiller_xa_hvar_parameters_minutes_hist::device_code, chiller_xa_hvar_parameters_minutes_hist::record_date))
        .do_update()
        .set((
            chiller_xa_hvar_parameters_minutes_hist::genunit_ui.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::genunit_ui)),
            chiller_xa_hvar_parameters_minutes_hist::cap_t.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::cap_t)),
            chiller_xa_hvar_parameters_minutes_hist::tot_curr.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::tot_curr)),
            chiller_xa_hvar_parameters_minutes_hist::ctrl_pnt.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::ctrl_pnt)),
            chiller_xa_hvar_parameters_minutes_hist::oat.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::oat)),
            chiller_xa_hvar_parameters_minutes_hist::cool_ewt.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::cool_ewt)),
            chiller_xa_hvar_parameters_minutes_hist::cool_lwt.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::cool_lwt)),
            chiller_xa_hvar_parameters_minutes_hist::circa_an_ui.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::circa_an_ui)),
            chiller_xa_hvar_parameters_minutes_hist::capa_t.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::capa_t)),
            chiller_xa_hvar_parameters_minutes_hist::dp_a.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::dp_a)),
            chiller_xa_hvar_parameters_minutes_hist::sp_a.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::sp_a)),
            chiller_xa_hvar_parameters_minutes_hist::econ_p_a.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::econ_p_a)),
            chiller_xa_hvar_parameters_minutes_hist::op_a.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::op_a)),
            chiller_xa_hvar_parameters_minutes_hist::dop_a.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::dop_a)),
            chiller_xa_hvar_parameters_minutes_hist::curren_a.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::curren_a)),
            chiller_xa_hvar_parameters_minutes_hist::cp_tmp_a.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::cp_tmp_a)),
            chiller_xa_hvar_parameters_minutes_hist::dgt_a.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::dgt_a)),
            chiller_xa_hvar_parameters_minutes_hist::eco_tp_a.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::eco_tp_a)),
            chiller_xa_hvar_parameters_minutes_hist::sct_a.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::sct_a)),
            chiller_xa_hvar_parameters_minutes_hist::sst_a.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::sst_a)),
            chiller_xa_hvar_parameters_minutes_hist::suct_t_a.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::suct_t_a)),
            chiller_xa_hvar_parameters_minutes_hist::exv_a.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::exv_a)),
            chiller_xa_hvar_parameters_minutes_hist::circb_an_ui.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::circb_an_ui)),
            chiller_xa_hvar_parameters_minutes_hist::capb_t.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::capb_t)),
            chiller_xa_hvar_parameters_minutes_hist::dp_b.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::dp_b)),
            chiller_xa_hvar_parameters_minutes_hist::sp_b.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::sp_b)),
            chiller_xa_hvar_parameters_minutes_hist::econ_p_b.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::econ_p_b)),
            chiller_xa_hvar_parameters_minutes_hist::op_b.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::op_b)),
            chiller_xa_hvar_parameters_minutes_hist::dop_b.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::dop_b)),
            chiller_xa_hvar_parameters_minutes_hist::curren_b.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::curren_b)),
            chiller_xa_hvar_parameters_minutes_hist::cp_tmp_b.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::cp_tmp_b)),
            chiller_xa_hvar_parameters_minutes_hist::dgt_b.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::dgt_b)),
            chiller_xa_hvar_parameters_minutes_hist::eco_tp_b.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::eco_tp_b)),
            chiller_xa_hvar_parameters_minutes_hist::sct_b.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::sct_b)),
            chiller_xa_hvar_parameters_minutes_hist::sst_b.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::sst_b)),
            chiller_xa_hvar_parameters_minutes_hist::suct_t_b.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::suct_t_b)),
            chiller_xa_hvar_parameters_minutes_hist::exv_b.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::exv_b)),
            chiller_xa_hvar_parameters_minutes_hist::circc_an_ui.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::circc_an_ui)),
            chiller_xa_hvar_parameters_minutes_hist::capc_t.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::capc_t)),
            chiller_xa_hvar_parameters_minutes_hist::dp_c.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::dp_c)),
            chiller_xa_hvar_parameters_minutes_hist::sp_c.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::sp_c)),
            chiller_xa_hvar_parameters_minutes_hist::econ_p_c.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::econ_p_c)),
            chiller_xa_hvar_parameters_minutes_hist::op_c.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::op_c)),
            chiller_xa_hvar_parameters_minutes_hist::dop_c.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::dop_c)),
            chiller_xa_hvar_parameters_minutes_hist::curren_c.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::curren_c)),
            chiller_xa_hvar_parameters_minutes_hist::cp_tmp_c.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::cp_tmp_c)),
            chiller_xa_hvar_parameters_minutes_hist::dgt_c.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::dgt_c)),
            chiller_xa_hvar_parameters_minutes_hist::eco_tp_c.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::eco_tp_c)),
            chiller_xa_hvar_parameters_minutes_hist::sct_c.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::sct_c)),
            chiller_xa_hvar_parameters_minutes_hist::sst_c.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::sst_c)),
            chiller_xa_hvar_parameters_minutes_hist::suct_t_c.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::suct_t_c)),
            chiller_xa_hvar_parameters_minutes_hist::exv_c.eq(excluded(chiller_xa_hvar_parameters_minutes_hist::exv_c)),
        ))
        .execute(&mut pool);

    match result {
        Ok(rows_affected) => {}
        Err(err) => {
            write_to_log_file_thread(&format!("Error inserting data in chiller_xa_hvar_parameters_minutes_hist, {:?}", err), 0, "ERROR");
            eprintln!("Inserting data error: {:?}, {}", data, err);
        }
    }
    
    drop(pool);
    
    Ok(())
}

pub fn get_chiller_xa_hvar_parameters_hist_minutes(device_code: &str, start_date: &str, end_date: &str, globs: &Arc<GlobalVars>) -> Result<Vec<ChillerXAHvarParametersHistRow>, Box<dyn Error>> {
    let mut pool = globs.pool.get()?;
    
    let params_hist = sql_query("SELECT 
        unit_id,
        device_code,
        genunit_ui,
        cap_t,
        tot_curr,
        ctrl_pnt,
        oat,
        cool_ewt,
        cool_lwt,
        circa_an_ui,
        capa_t,
        dp_a,
        sp_a,
        econ_p_a,
        op_a,
        dop_a,
        curren_a,
        cp_tmp_a,
        dgt_a,
        eco_tp_a,
        sct_a,
        sst_a,
        sst_b,
        suct_t_a,
        exv_a,
        circb_an_ui,
        capb_t,
        dp_b,
        sp_b,
        econ_p_b,
        op_b,
        dop_b,
        curren_b,
        cp_tmp_b,
        dgt_b,
        eco_tp_b,
        sct_b,
        suct_t_b,
        exv_b,
        circc_an_ui,
        capc_t,
        dp_c,
        sp_c,
        econ_p_c,
        op_c,
        dop_c,
        curren_c,
        cp_tmp_c,
        dgt_c,
        eco_tp_c,
        sct_c,
        sst_c,
        suct_t_c,
        exv_c,
        record_date
    FROM
        chiller_xa_hvar_parameters_minutes_hist
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

    let response = params_hist.load::<ChillerXAHvarParametersHistRow>(&mut pool)?;

    Ok(response)
}

pub fn get_chiller_xa_hvar_parameters_hist_hour(device_code: &str, start_date: &str, end_date: &str, globs: &Arc<GlobalVars>) -> Result<Vec<ChillerXAHvarParametersHistRow>, Box<dyn Error>> {
    let mut pool = globs.pool.get()?;
    
    let params_hist = sql_query("SELECT 
        unit_id,
        device_code,
        genunit_ui,
        cap_t,
        tot_curr,
        ctrl_pnt,
        oat,
        cool_ewt,
        cool_lwt,
        circa_an_ui,
        capa_t,
        dp_a,
        sp_a,
        econ_p_a,
        op_a,
        dop_a,
        curren_a,
        cp_tmp_a,
        dgt_a,
        eco_tp_a,
        sct_a,
        sst_a,
        sst_b,
        suct_t_a,
        exv_a,
        circb_an_ui,
        capb_t,
        dp_b,
        sp_b,
        econ_p_b,
        op_b,
        dop_b,
        curren_b,
        cp_tmp_b,
        dgt_b,
        eco_tp_b,
        sct_b,
        suct_t_b,
        exv_b,
        circc_an_ui,
        capc_t,
        dp_c,
        sp_c,
        econ_p_c,
        op_c,
        dop_c,
        curren_c,
        cp_tmp_c,
        dgt_c,
        eco_tp_c,
        sct_c,
        sst_c,
        suct_t_c,
        exv_c,
        compilation_record_date as record_date
    FROM
        chiller_xa_hvar_parameters_hist_view
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

    let response = params_hist.load::<ChillerXAHvarParametersHistRow>(&mut pool)?;

    Ok(response)
}
