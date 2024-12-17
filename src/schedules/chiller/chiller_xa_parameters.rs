use std::{collections::HashMap, sync::Arc};

use chrono::{Duration, Local, NaiveDateTime, Timelike};
use rust_decimal::{prelude::FromPrimitive, Decimal};

use crate::{compression::common_func::check_amount_minutes_offline, db::entities::chiller::{chiller_parameters_changes_hist::insert_data_change_parameters_hist, chiller_xa_hvar_parameters_minutes_hist::insert_chiller_xa_hvar_parameters_hist, chiller_xa_parameters_minutes_hist::insert_chiller_xa_parameters_hist}, models::{database_models::chiller::{chiller_parameters_changes_hist::ChillerParametersChangesHist, chiller_xa_hvar_parameters_minutes_hist::ChillerXAHvarParametersMinutesHist, chiller_xa_parameters_minute_hist::ChillerXAParametersMinutesHist}, external_models::device::DriDevice}, schedules::{device_disponibility::insert_device_disponibility_hist, devices::{process_chiller_xa_devices_by_script, process_chiller_xa_hvar_devices_by_script}}, telemetry_payloads::dri_telemetry::{DriChillerCarrierXAChangeParams, DriChillerCarrierXAHvarChangeParams, DriChillerCarrierXAHvarTelemetry, DriChillerCarrierXATelemetry}, GlobalVars};

#[derive(Debug)]
pub struct SimpleTelemetry {
    pub timestamp: NaiveDateTime,
}

pub fn group_telemetries_by_10_minutes_xa(device_code: &str, unit_id: i32, telemetries: Vec<DriChillerCarrierXATelemetry>, globs: &Arc<GlobalVars>) -> HashMap<NaiveDateTime, Vec<DriChillerCarrierXATelemetry>> {
    let mut grouped_telemetries: HashMap<NaiveDateTime, Vec<DriChillerCarrierXATelemetry>> = HashMap::new();
    let mut last_telemetry = DriChillerCarrierXAChangeParams::new(Local::now().naive_local());

    for (index, telemetry) in telemetries.iter().enumerate() {
        let timestamp = telemetry.timestamp; 
        let rounded_minute = ((timestamp.minute() / 10) * 10) as u32;
        let rounded_timestamp = timestamp.date().and_hms(timestamp.hour(), rounded_minute, 0);
        grouped_telemetries.entry(rounded_timestamp).or_insert_with(Vec::new).push(telemetry.clone());

        let iterate_telemetry = DriChillerCarrierXAChangeParams {
            timestamp: telemetry.timestamp,
            STATUS: telemetry.STATUS,
            CHIL_S_S: telemetry.CHIL_S_S,
            CHIL_OCC: telemetry.CHIL_OCC,
            CTRL_TYP: telemetry.CTRL_TYP,
            SLC_HM: telemetry.SLC_HM,
            DEM_LIM: telemetry.DEM_LIM,
            SP_OCC: telemetry.SP_OCC,
            EMSTOP: telemetry.EMSTOP,
        };

        if index == 0 || index == telemetries.len() - 1 {
            // primeira telemetria ou última, salvar dados
            salve_all_params_change(device_code, unit_id, iterate_telemetry, globs);
        } else {
            verify_and_save_params_changes(device_code, unit_id, iterate_telemetry, last_telemetry.clone(), globs);
        }

        last_telemetry.timestamp = telemetry.timestamp;
        last_telemetry.STATUS = telemetry.STATUS;
        last_telemetry.CHIL_S_S = telemetry.CHIL_S_S;
        last_telemetry.CHIL_OCC = telemetry.CHIL_OCC;
        last_telemetry.CTRL_TYP = telemetry.CTRL_TYP;
        last_telemetry.SLC_HM = telemetry.SLC_HM;
        last_telemetry.DEM_LIM = telemetry.DEM_LIM;
        last_telemetry.SP_OCC = telemetry.SP_OCC;
        last_telemetry.EMSTOP = telemetry.EMSTOP;
    }

    grouped_telemetries
}

fn verify_and_save_params_changes(device_code: &str, unit_id: i32, iterate_telemetry: DriChillerCarrierXAChangeParams, last_telemetry: DriChillerCarrierXAChangeParams, globs: &Arc<GlobalVars>) {
    save_if_changed("STATUS", &last_telemetry.STATUS, iterate_telemetry.STATUS, iterate_telemetry.timestamp, device_code, unit_id, globs);
    save_if_changed("CHIL_S_S", &last_telemetry.CHIL_S_S, iterate_telemetry.CHIL_S_S, iterate_telemetry.timestamp, device_code, unit_id, globs);
    save_if_changed("CHIL_OCC", &last_telemetry.CHIL_OCC, iterate_telemetry.CHIL_OCC, iterate_telemetry.timestamp, device_code, unit_id, globs);
    save_if_changed("CTRL_TYP", &last_telemetry.CTRL_TYP, iterate_telemetry.CTRL_TYP, iterate_telemetry.timestamp, device_code, unit_id, globs);
    save_if_changed("SLC_HM", &last_telemetry.SLC_HM, iterate_telemetry.SLC_HM, iterate_telemetry.timestamp, device_code, unit_id, globs);
    save_if_changed("DEM_LIM", &last_telemetry.DEM_LIM, iterate_telemetry.DEM_LIM, iterate_telemetry.timestamp, device_code, unit_id, globs);
    save_if_changed("SP_OCC", &last_telemetry.SP_OCC, iterate_telemetry.SP_OCC, iterate_telemetry.timestamp, device_code, unit_id, globs);
    save_if_changed("EMSTOP", &last_telemetry.EMSTOP, iterate_telemetry.EMSTOP, iterate_telemetry.timestamp, device_code, unit_id, globs);
}

fn save_if_changed(param_name: &str, last_value: &Option<f64>, new_value: Option<f64>, timestamp: NaiveDateTime, device_code: &str, unit_id: i32, globs: &Arc<GlobalVars>) {
    if let Some(new_val) = new_value {
        if last_value != &Some(new_val) {
            save_param_change_hist(timestamp, device_code, unit_id, param_name, Some(new_val), globs);
        }
    }
}

fn salve_all_params_change(device_code: &str, unit_id: i32, last_telemetry: DriChillerCarrierXAChangeParams, globs: &Arc<GlobalVars>) {
    save_param_change_hist(last_telemetry.timestamp, device_code, unit_id, "STATUS", last_telemetry.STATUS, globs);
    save_param_change_hist(last_telemetry.timestamp, device_code, unit_id, "CHIL_S_S", last_telemetry.CHIL_S_S, globs);
    save_param_change_hist(last_telemetry.timestamp, device_code, unit_id, "CHIL_OCC", last_telemetry.CHIL_OCC, globs);
    save_param_change_hist(last_telemetry.timestamp, device_code, unit_id, "CTRL_TYP", last_telemetry.CTRL_TYP, globs);
    save_param_change_hist(last_telemetry.timestamp, device_code, unit_id, "SLC_HM", last_telemetry.SLC_HM, globs);
    save_param_change_hist(last_telemetry.timestamp, device_code, unit_id, "DEM_LIM", last_telemetry.DEM_LIM, globs);
    save_param_change_hist(last_telemetry.timestamp, device_code, unit_id, "SP_OCC", last_telemetry.SP_OCC, globs);
    save_param_change_hist(last_telemetry.timestamp, device_code, unit_id, "EMSTOP", last_telemetry.EMSTOP, globs);
}

fn save_param_change_hist(day: NaiveDateTime, device_code: &str, unit_id: i32, parameter_name: &str, parameter_value: Option<f64>, globs: &Arc<GlobalVars>) {
    if let Some(value) = parameter_value {
        let history = ChillerParametersChangesHist {
            unit_id,
            device_code: String::from(device_code),
            parameter_name: parameter_name.to_string(),
            record_date: day,
            parameter_value: value.round() as i32,
        };

        insert_data_change_parameters_hist(history, globs);
    }
}

pub fn calculate_group_averages_xa(grouped_telemetries: &HashMap<NaiveDateTime, Vec<DriChillerCarrierXATelemetry>>) -> Vec<DriChillerCarrierXATelemetry> {
    let mut group_averages: Vec<DriChillerCarrierXATelemetry> = Vec::new();

    for (time_interval, telemetry_array) in grouped_telemetries {
        let mut total_values: HashMap<&str, (f64, usize)> = HashMap::new();

        for telemetry in telemetry_array {
            for (field, value) in &[
                ("CAP_T", telemetry.CAP_T),
                ("COND_EWT", telemetry.COND_EWT),
                ("COND_LWT", telemetry.COND_LWT),
                ("COOL_EWT", telemetry.COOL_EWT),
                ("COOL_LWT", telemetry.COOL_LWT),
                ("CTRL_PNT", telemetry.CTRL_PNT),
                ("DP_A", telemetry.DP_A),
                ("DP_B", telemetry.DP_B),
                ("HR_CP_A", telemetry.HR_CP_A),
                ("HR_CP_B", telemetry.HR_CP_B),
                ("HR_MACH", telemetry.HR_MACH),
                ("HR_MACH_B", telemetry.HR_MACH_B),
                ("OAT", telemetry.OAT),
                ("OP_A", telemetry.OP_A),
                ("OP_B", telemetry.OP_B),
                ("SCT_A", telemetry.SCT_A),
                ("SCT_B", telemetry.SCT_B),
                ("SLT_A", telemetry.SLT_A),
                ("SLT_B", telemetry.SLT_B),
                ("SP", telemetry.SP),
                ("SP_A", telemetry.SP_A),
                ("SP_B", telemetry.SP_B),
                ("SST_A", telemetry.SST_A),
                ("SST_B", telemetry.SST_B),
            ] {
                if let Some(v) = value {
                    let (sum, count_val) = total_values.entry(field).or_insert((0.0, 0));
                    *sum += v;
                    *count_val += 1;
                }
            }
        }
        let mut averaged_telemetry = DriChillerCarrierXATelemetry::new(*time_interval);

        for (field, (total, field_count)) in total_values {
            let avg = total / field_count as f64;
            let avg_rounded = (avg * 100.0).round() / 100.0;
            averaged_telemetry.set_field_average(field, avg_rounded);
        }

        group_averages.push(averaged_telemetry);
    }

    group_averages
}

pub fn insert_chiller_xa_parameters(hist: Vec<DriChillerCarrierXATelemetry>, device_code: &str, unit_id: i32, globs: &Arc<GlobalVars>) {
    for minute_hist in &hist {
        let history = ChillerXAParametersMinutesHist {
            unit_id,
            device_code: device_code.to_string(),
            cap_t: Decimal::from_f64(minute_hist.CAP_T.unwrap_or(0.0)).unwrap(),
            cond_ewt: Decimal::from_f64(minute_hist.COND_EWT.unwrap_or(0.0)).unwrap(),
            cond_lwt: Decimal::from_f64(minute_hist.COND_LWT.unwrap_or(0.0)).unwrap(),
            cool_ewt: Decimal::from_f64(minute_hist.COOL_EWT.unwrap_or(0.0)).unwrap(),
            cool_lwt: Decimal::from_f64(minute_hist.COOL_LWT.unwrap_or(0.0)).unwrap(),
            ctrl_pnt: Decimal::from_f64(minute_hist.CTRL_PNT.unwrap_or(0.0)).unwrap(),
            dp_a: Decimal::from_f64(minute_hist.DP_A.unwrap_or(0.0)).unwrap(),
            dp_b: Decimal::from_f64(minute_hist.DP_B.unwrap_or(0.0)).unwrap(),
            hr_cp_a: Decimal::from_f64(minute_hist.HR_CP_A.unwrap_or(0.0)).unwrap(),
            hr_cp_b: Decimal::from_f64(minute_hist.HR_CP_B.unwrap_or(0.0)).unwrap(),
            hr_mach: Decimal::from_f64(minute_hist.HR_MACH.unwrap_or(0.0)).unwrap(),
            hr_mach_b: Decimal::from_f64(minute_hist.HR_MACH_B.unwrap_or(0.0)).unwrap(),
            oat: Decimal::from_f64(minute_hist.OAT.unwrap_or(0.0)).unwrap(),
            op_a: Decimal::from_f64(minute_hist.OP_A.unwrap_or(0.0)).unwrap(),
            op_b: Decimal::from_f64(minute_hist.OP_B.unwrap_or(0.0)).unwrap(),
            sct_a: Decimal::from_f64(minute_hist.SCT_A.unwrap_or(0.0)).unwrap(),
            sct_b: Decimal::from_f64(minute_hist.SCT_B.unwrap_or(0.0)).unwrap(),
            slt_a: Decimal::from_f64(minute_hist.SLT_A.unwrap_or(0.0)).unwrap(),
            slt_b: Decimal::from_f64(minute_hist.SLT_B.unwrap_or(0.0)).unwrap(),
            sp: Decimal::from_f64(minute_hist.SP.unwrap_or(0.0)).unwrap(),
            sp_a: Decimal::from_f64(minute_hist.SP_A.unwrap_or(0.0)).unwrap(),
            sp_b: Decimal::from_f64(minute_hist.SP_B.unwrap_or(0.0)).unwrap(),
            sst_a: Decimal::from_f64(minute_hist.SST_A.unwrap_or(0.0)).unwrap(),
            sst_b: Decimal::from_f64(minute_hist.SST_B.unwrap_or(0.0)).unwrap(),
            record_date: minute_hist.timestamp,
        };

        insert_chiller_xa_parameters_hist(history, globs);
    }
}

pub async fn process_chiller_xa_devices(unit_id: i32, chiller_devices: &Option<Vec<DriDevice>>, day: &str, client_minutes_to_check_offline: Option<i32>, globs: &Arc<GlobalVars>) {
    if let Some(devices) = chiller_devices {
        process_chiller_xa_devices_by_script(unit_id, day, &devices, client_minutes_to_check_offline, globs).await;
    }
}

pub async fn process_chiller_xa_hvar_devices(unit_id: i32, chiller_devices: &Option<Vec<DriDevice>>, day: &str, client_minutes_to_check_offline: Option<i32>, globs: &Arc<GlobalVars>) {
    if let Some(devices) = chiller_devices {
        process_chiller_xa_hvar_devices_by_script(unit_id, day, &devices, client_minutes_to_check_offline, globs).await;
    }
}

pub fn verify_hours_online_xa(chiller_hist: Vec<SimpleTelemetry>, client_minutes_to_check_offline: Option<i32>, dri_interval: Option<isize>, start_date: NaiveDateTime, device_code: &str, unit_id: i32, day: &str, globs: &Arc<GlobalVars>) {
    let mut v_timestamp: Vec<String> = Vec::new();
    let end_date =  start_date + Duration::days(1);
    let interval = dri_interval.unwrap_or(300) as i32;
    for hist in chiller_hist {
        if hist.timestamp < end_date {
                v_timestamp.push(hist.timestamp.format("%Y-%m-%dT%H:%M:%S").to_string().clone());
        }
    }

    let mut hours_online = 0.0;

    if let Some(minutes_to_check) = client_minutes_to_check_offline {
        hours_online = check_amount_minutes_offline(minutes_to_check, v_timestamp, &start_date.format("%Y-%m-%dT%H:%M:%S").to_string());

    } else {
        let minutes = interval / 60;
        hours_online = check_amount_minutes_offline(minutes, v_timestamp, &start_date.format("%Y-%m-%dT%H:%M:%S").to_string());
    }

    insert_device_disponibility_hist(unit_id, Decimal::from_f64_retain(hours_online).unwrap_or(Decimal::new(0,0)), day, device_code, globs);
}

pub fn convert_to_simple_xa(chiller_hist: Vec<DriChillerCarrierXATelemetry>) -> Vec<SimpleTelemetry> {
    chiller_hist
        .into_iter()
        .map(|hist| SimpleTelemetry {
            timestamp: hist.timestamp,
        })
        .collect()
}

pub fn convert_to_simple_xa_hvar(chiller_hist: Vec<DriChillerCarrierXAHvarTelemetry>) -> Vec<SimpleTelemetry> {
    chiller_hist
        .into_iter()
        .map(|hist| SimpleTelemetry {
            timestamp: hist.timestamp,
        })
        .collect()
}

pub fn insert_chiller_xa_hvar_parameters(hist: Vec<DriChillerCarrierXAHvarTelemetry>, device_code: &str, unit_id: i32, globs: &Arc<GlobalVars>) {
    for minute_hist in &hist {
        let history = ChillerXAHvarParametersMinutesHist {
            unit_id,
            device_code: device_code.to_string(),
            genunit_ui: get_value_decimal(minute_hist.GENUNIT_UI),
            cap_t: get_value_decimal(minute_hist.CAP_T),
            tot_curr: get_value_decimal(minute_hist.TOT_CURR),
            ctrl_pnt: get_value_decimal(minute_hist.CTRL_PNT),
            record_date: minute_hist.timestamp,
            cool_ewt: get_value_decimal(minute_hist.COOL_EWT),
            cool_lwt: get_value_decimal(minute_hist.COOL_LWT),
            circa_an_ui: get_value_decimal(minute_hist.CIRCA_AN_UI),
            capa_t: get_value_decimal(minute_hist.CAPA_T),
            dp_a: get_value_decimal(minute_hist.DP_A),
            sp_a: get_value_decimal(minute_hist.SP_A),
            econ_p_a: get_value_decimal(minute_hist.ECON_P_A),
            op_a: get_value_decimal(minute_hist.OP_A),
            dop_a: get_value_decimal(minute_hist.DOP_A),
            curren_a: get_value_decimal(minute_hist.CURREN_A),
            cp_tmp_a: get_value_decimal(minute_hist.CP_TMP_A),
            dgt_a: get_value_decimal(minute_hist.DGT_A),
            eco_tp_a: get_value_decimal(minute_hist.ECO_TP_A),
            sct_a: get_value_decimal(minute_hist.SCT_A),
            sst_a: get_value_decimal(minute_hist.SST_A),
            sst_b: get_value_decimal(minute_hist.SST_B),
            suct_t_a: get_value_decimal(minute_hist.SUCT_T_A),
            exv_a: get_value_decimal(minute_hist.EXV_A),
            circb_an_ui: get_value_decimal(minute_hist.CIRCB_AN_UI),
            capb_t: get_value_decimal(minute_hist.CAPB_T),
            dp_b: get_value_decimal(minute_hist.DP_B),
            sp_b: get_value_decimal(minute_hist.SP_B),
            econ_p_b: get_value_decimal(minute_hist.ECON_P_B),
            op_b: get_value_decimal(minute_hist.OP_B),
            dop_b: get_value_decimal(minute_hist.DOP_B),
            curren_b: get_value_decimal(minute_hist.CURREN_B),
            cp_tmp_b: get_value_decimal(minute_hist.CP_TMP_B),
            dgt_b: get_value_decimal(minute_hist.DGT_B),
            eco_tp_b: get_value_decimal(minute_hist.ECO_TP_B),
            sct_b: get_value_decimal(minute_hist.SCT_B),
            suct_t_b: get_value_decimal(minute_hist.SUCT_T_B),
            exv_b: get_value_decimal(minute_hist.EXV_B),
            circc_an_ui: get_value_decimal(minute_hist.CIRCC_AN_UI),
            capc_t: get_value_decimal(minute_hist.CAPC_T),
            dp_c: get_value_decimal(minute_hist.DP_C),
            sp_c: get_value_decimal(minute_hist.SP_C),
            econ_p_c: get_value_decimal(minute_hist.ECON_P_C),
            op_c: get_value_decimal(minute_hist.OP_C),
            dop_c: get_value_decimal(minute_hist.DOP_C),
            curren_c: get_value_decimal(minute_hist.CURREN_C),
            cp_tmp_c: get_value_decimal(minute_hist.CP_TMP_C),
            dgt_c: get_value_decimal(minute_hist.DGT_C),
            eco_tp_c: get_value_decimal(minute_hist.ECO_TP_C),
            sct_c: get_value_decimal(minute_hist.SCT_C),
            sst_c: get_value_decimal(minute_hist.SST_C),
            suct_t_c: get_value_decimal(minute_hist.SUCT_T_C),
            exv_c: get_value_decimal(minute_hist.EXV_C),
            oat: get_value_decimal(minute_hist.OAT),
        };

        insert_chiller_xa_hvar_parameters_hist(history, globs);
    }
}

fn get_value_decimal(val: Option<f64>) -> Decimal {
    Decimal::from_f64(val.unwrap_or(0.0)).unwrap_or(Decimal::new(0, 0))
}

fn verify_and_save_params_changes_hvar(device_code: &str, unit_id: i32, iterate_telemetry: DriChillerCarrierXAHvarChangeParams, last_telemetry: DriChillerCarrierXAHvarChangeParams, globs: &Arc<GlobalVars>) {
    save_if_changed("STATUS", &last_telemetry.STATUS, iterate_telemetry.STATUS, iterate_telemetry.timestamp, device_code, unit_id, globs);
    save_if_changed("CHIL_S_S", &last_telemetry.CHIL_S_S, iterate_telemetry.CHIL_S_S, iterate_telemetry.timestamp, device_code, unit_id, globs);
    save_if_changed("CHIL_OCC", &last_telemetry.CHIL_OCC, iterate_telemetry.CHIL_OCC, iterate_telemetry.timestamp, device_code, unit_id, globs);
    save_if_changed("CTRL_TYP", &last_telemetry.CTRL_TYP, iterate_telemetry.CTRL_TYP, iterate_telemetry.timestamp, device_code, unit_id, globs);
    save_if_changed("ALM", &last_telemetry.ALM, iterate_telemetry.ALM, iterate_telemetry.timestamp, device_code, unit_id, globs);
    save_if_changed("DEM_LIM", &last_telemetry.DEM_LIM, iterate_telemetry.DEM_LIM, iterate_telemetry.timestamp, device_code, unit_id, globs);
    save_if_changed("SP_OCC", &last_telemetry.SP_OCC, iterate_telemetry.SP_OCC, iterate_telemetry.timestamp, device_code, unit_id, globs);
    save_if_changed("EMSTOP", &last_telemetry.EMSTOP, iterate_telemetry.EMSTOP, iterate_telemetry.timestamp, device_code, unit_id, globs);
}

fn salve_all_params_change_hvar(device_code: &str, unit_id: i32, last_telemetry: DriChillerCarrierXAHvarChangeParams, globs: &Arc<GlobalVars>) {
    save_param_change_hist(last_telemetry.timestamp, device_code, unit_id, "STATUS", last_telemetry.STATUS, globs);
    save_param_change_hist(last_telemetry.timestamp, device_code, unit_id, "CHIL_S_S", last_telemetry.CHIL_S_S, globs);
    save_param_change_hist(last_telemetry.timestamp, device_code, unit_id, "CHIL_OCC", last_telemetry.CHIL_OCC, globs);
    save_param_change_hist(last_telemetry.timestamp, device_code, unit_id, "CTRL_TYP", last_telemetry.CTRL_TYP, globs);
    save_param_change_hist(last_telemetry.timestamp, device_code, unit_id, "ALM", last_telemetry.ALM, globs);
    save_param_change_hist(last_telemetry.timestamp, device_code, unit_id, "DEM_LIM", last_telemetry.DEM_LIM, globs);
    save_param_change_hist(last_telemetry.timestamp, device_code, unit_id, "SP_OCC", last_telemetry.SP_OCC, globs);
    save_param_change_hist(last_telemetry.timestamp, device_code, unit_id, "EMSTOP", last_telemetry.EMSTOP, globs);
}

pub fn group_telemetries_by_10_minutes_xa_hvar(device_code: &str, unit_id: i32, telemetries: Vec<DriChillerCarrierXAHvarTelemetry>, globs: &Arc<GlobalVars>) -> HashMap<NaiveDateTime, Vec<DriChillerCarrierXAHvarTelemetry>> {
    let mut grouped_telemetries: HashMap<NaiveDateTime, Vec<DriChillerCarrierXAHvarTelemetry>> = HashMap::new();
    let mut last_telemetry = DriChillerCarrierXAHvarChangeParams::new(Local::now().naive_local());

    for (index, telemetry) in telemetries.iter().enumerate() {
        let timestamp = telemetry.timestamp; 
        let rounded_minute = ((timestamp.minute() / 10) * 10) as u32;
        let rounded_timestamp = timestamp.date().and_hms(timestamp.hour(), rounded_minute, 0);
        grouped_telemetries.entry(rounded_timestamp).or_insert_with(Vec::new).push(telemetry.clone());

        let iterate_telemetry = DriChillerCarrierXAHvarChangeParams {
            timestamp: telemetry.timestamp,
            STATUS: telemetry.STATUS,
            CHIL_S_S: telemetry.CHIL_S_S,
            CHIL_OCC: telemetry.CHIL_OCC,
            CTRL_TYP: telemetry.CTRL_TYP,
            DEM_LIM: telemetry.DEM_LIM,
            SP_OCC: telemetry.SP_OCC,
            EMSTOP: telemetry.EMSTOP,
            ALM: telemetry.ALM,
        };

        if index == 0 || index == telemetries.len() - 1 {
            // primeira telemetria ou última, salvar dados
            salve_all_params_change_hvar(device_code, unit_id, iterate_telemetry, globs);
        } else {
            verify_and_save_params_changes_hvar(device_code, unit_id, iterate_telemetry, last_telemetry.clone(), globs);
        }

        last_telemetry.timestamp = telemetry.timestamp;
        last_telemetry.STATUS = telemetry.STATUS;
        last_telemetry.CHIL_S_S = telemetry.CHIL_S_S;
        last_telemetry.CHIL_OCC = telemetry.CHIL_OCC;
        last_telemetry.CTRL_TYP = telemetry.CTRL_TYP;
        last_telemetry.ALM = telemetry.ALM;
        last_telemetry.DEM_LIM = telemetry.DEM_LIM;
        last_telemetry.SP_OCC = telemetry.SP_OCC;
        last_telemetry.EMSTOP = telemetry.EMSTOP;
    }

    grouped_telemetries
}

pub fn calculate_group_averages_xa_hvar(grouped_telemetries: &HashMap<NaiveDateTime, Vec<DriChillerCarrierXAHvarTelemetry>>) -> Vec<DriChillerCarrierXAHvarTelemetry> {
    let mut group_averages: Vec<DriChillerCarrierXAHvarTelemetry> = Vec::new();

    for (time_interval, telemetry_array) in grouped_telemetries {
        let mut total_values: HashMap<&str, (f64, usize)> = HashMap::new();

        for telemetry in telemetry_array {
            for (field, value) in &[
                ("GENUNIT_UI", telemetry.GENUNIT_UI),
                ("CAP_T", telemetry.CAP_T),
                ("TOT_CURR", telemetry.TOT_CURR),
                ("CTRL_PNT", telemetry.CTRL_PNT),
                ("OAT", telemetry.OAT),
                ("COOL_EWT", telemetry.COOL_EWT),
                ("COOL_LWT", telemetry.COOL_LWT),
                ("CIRCA_AN_UI", telemetry.CIRCA_AN_UI),
                ("CAPA_T", telemetry.CAPA_T),
                ("DP_A", telemetry.DP_A),
                ("SP_A", telemetry.SP_A),
                ("ECON_P_A", telemetry.ECON_P_A),
                ("OP_A", telemetry.OP_A),
                ("DOP_A", telemetry.DOP_A),
                ("CURREN_A", telemetry.CURREN_A),
                ("CP_TMP_A", telemetry.CP_TMP_A),
                ("DGT_A", telemetry.DGT_A),
                ("ECO_TP_A", telemetry.ECO_TP_A),
                ("SCT_A", telemetry.SCT_A),
                ("SST_A", telemetry.SST_A),
                ("SST_B", telemetry.SST_B),
                ("SUCT_T_A", telemetry.SUCT_T_A),
                ("EXV_A", telemetry.EXV_A),
                ("CIRCB_AN_UI", telemetry.CIRCB_AN_UI),
                ("CAPB_T", telemetry.CAPB_T),
                ("DP_B", telemetry.DP_B),
                ("SP_B", telemetry.SP_B),
                ("ECON_P_B", telemetry.ECON_P_B),
                ("OP_B", telemetry.OP_B),
                ("DOP_B", telemetry.DOP_B),
                ("CURREN_B", telemetry.CURREN_B),
                ("CP_TMP_B", telemetry.CP_TMP_B),
                ("DGT_B", telemetry.DGT_B),
                ("ECO_TP_B", telemetry.ECO_TP_B),
                ("SCT_B", telemetry.SCT_B),
                ("SUCT_T_B", telemetry.SUCT_T_B),
                ("EXV_B", telemetry.EXV_B),
                ("CIRCC_AN_UI", telemetry.CIRCC_AN_UI),
                ("CAPC_T", telemetry.CAPC_T),
                ("DP_C", telemetry.DP_C),
                ("SP_C", telemetry.SP_C),
                ("ECON_P_C", telemetry.ECON_P_C),
                ("OP_C", telemetry.OP_C),
                ("DOP_C", telemetry.DOP_C),
                ("CURREN_C", telemetry.CURREN_C),
                ("CP_TMP_C", telemetry.CP_TMP_C),
                ("DGT_C", telemetry.DGT_C),
                ("ECO_TP_C", telemetry.ECO_TP_C),
                ("SCT_C", telemetry.SCT_C),
                ("SST_C", telemetry.SST_C),
                ("SUCT_T_C", telemetry.SUCT_T_C),
                ("EXV_C", telemetry.EXV_C),
            ] {
                if let Some(v) = value {
                    let (sum, count_val) = total_values.entry(field).or_insert((0.0, 0));
                    *sum += v;
                    *count_val += 1;
                }
            }
        }
        let mut averaged_telemetry = DriChillerCarrierXAHvarTelemetry::new(*time_interval);

        for (field, (total, field_count)) in total_values {
            let avg = total / field_count as f64;
            let avg_rounded = (avg * 100.0).round() / 100.0;
            averaged_telemetry.set_field_average(field, avg_rounded);
        }

        group_averages.push(averaged_telemetry);
    }

    group_averages
}
