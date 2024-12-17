use std::{collections::HashMap, sync::Arc};

use chrono::{Duration, Local, NaiveDateTime, Timelike};
use rust_decimal::{prelude::FromPrimitive, Decimal};

use crate::{compression::common_func::check_amount_minutes_offline, db::entities::chiller::{chiller_hx_parameters_minutes_hist::insert_chiller_hx_parameters_hist, chiller_parameters_changes_hist::insert_data_change_parameters_hist}, models::{database_models::chiller::{chiller_hx_parameters_minute_hist::ChillerHXParametersMinutesHist, chiller_parameters_changes_hist::ChillerParametersChangesHist}, external_models::device::DriDevice}, schedules::{device_disponibility::insert_device_disponibility_hist, devices::process_chiller_hx_devices_by_script}, telemetry_payloads::dri_telemetry::{DriChillerCarrierChangeParams, DriChillerCarrierHXTelemetry}, GlobalVars};

pub fn group_telemetries_by_10_minutes_hx(device_code: &str, unit_id: i32, telemetries: Vec<DriChillerCarrierHXTelemetry>, globs: &Arc<GlobalVars>) -> HashMap<NaiveDateTime, Vec<DriChillerCarrierHXTelemetry>> {
    let mut grouped_telemetries: HashMap<NaiveDateTime, Vec<DriChillerCarrierHXTelemetry>> = HashMap::new();
    let mut last_telemetry = DriChillerCarrierChangeParams::new(Local::now().naive_local());

    for (index, telemetry) in telemetries.iter().enumerate() {
        let timestamp = telemetry.timestamp; 
        let rounded_minute = ((timestamp.minute() / 10) * 10) as u32;
        let rounded_timestamp = timestamp.date().and_hms(timestamp.hour(), rounded_minute, 0);
        grouped_telemetries.entry(rounded_timestamp).or_insert_with(Vec::new).push(telemetry.clone());

        let iterate_telemetry = DriChillerCarrierChangeParams {
            timestamp: telemetry.timestamp,
            CHIL_S_S: telemetry.CHIL_S_S,
            ALM: telemetry.ALM,
            EMSTOP: telemetry.EMSTOP,
            STATUS: telemetry.STATUS,
            CHIL_OCC: telemetry.CHIL_OCC,
            CP_A1: telemetry.CP_A1,
            CP_A2: telemetry.CP_A2,
            CP_B1: telemetry.CP_B1,
            CP_B2: telemetry.CP_B2,
        };

        if index == 0 || index == telemetries.len() - 1 {
            // primeira telemetria ou última, salvar dados
            salve_all_params_change(device_code, unit_id, iterate_telemetry, globs);
        } else {
            verify_and_save_params_changes(device_code, unit_id, iterate_telemetry, last_telemetry.clone(), globs);
        }

        last_telemetry.timestamp = telemetry.timestamp;
        last_telemetry.CHIL_S_S = telemetry.CHIL_S_S;
        last_telemetry.ALM = telemetry.ALM;
        last_telemetry.EMSTOP = telemetry.EMSTOP;
        last_telemetry.STATUS = telemetry.STATUS;
        last_telemetry.CHIL_OCC = telemetry.CHIL_OCC;
        last_telemetry.CP_A1 = telemetry.CP_A1;
        last_telemetry.CP_A2 = telemetry.CP_A2;
        last_telemetry.CP_B1 = telemetry.CP_B1;
        last_telemetry.CP_B2 = telemetry.CP_B2;
    }

    grouped_telemetries
}

fn verify_and_save_params_changes(device_code: &str, unit_id: i32, iterate_telemetry: DriChillerCarrierChangeParams, last_telemetry: DriChillerCarrierChangeParams, globs: &Arc<GlobalVars>) {
    save_if_changed("CHIL_S_S", &last_telemetry.CHIL_S_S, iterate_telemetry.CHIL_S_S, iterate_telemetry.timestamp, device_code, unit_id, globs);
    save_if_changed("EMSTOP", &last_telemetry.EMSTOP, iterate_telemetry.EMSTOP, iterate_telemetry.timestamp, device_code, unit_id, globs);
    save_if_changed("ALM", &last_telemetry.ALM, iterate_telemetry.ALM, iterate_telemetry.timestamp, device_code, unit_id, globs);
    save_if_changed("STATUS", &last_telemetry.STATUS, iterate_telemetry.STATUS, iterate_telemetry.timestamp, device_code, unit_id, globs);
    save_if_changed("CHIL_OCC", &last_telemetry.CHIL_OCC, iterate_telemetry.CHIL_OCC, iterate_telemetry.timestamp, device_code, unit_id, globs);
    save_if_changed("CP_A1", &last_telemetry.CP_A1, iterate_telemetry.CP_A1, iterate_telemetry.timestamp, device_code, unit_id, globs);
    save_if_changed("CP_A2", &last_telemetry.CP_A2, iterate_telemetry.CP_A2, iterate_telemetry.timestamp, device_code, unit_id, globs);
    save_if_changed("CP_B1", &last_telemetry.CP_B1, iterate_telemetry.CP_B1, iterate_telemetry.timestamp, device_code, unit_id, globs);
    save_if_changed("CP_B2", &last_telemetry.CP_B2, iterate_telemetry.CP_B2, iterate_telemetry.timestamp, device_code, unit_id, globs);
}

fn save_if_changed(param_name: &str, last_value: &Option<f64>, new_value: Option<f64>, timestamp: NaiveDateTime, device_code: &str, unit_id: i32, globs: &Arc<GlobalVars>) {
    if let Some(new_val) = new_value {
        if last_value != &Some(new_val) {
            save_param_change_hist(timestamp, device_code, unit_id, param_name, Some(new_val), globs);
        }
    }
}

fn salve_all_params_change(device_code: &str, unit_id: i32, last_telemetry: DriChillerCarrierChangeParams, globs: &Arc<GlobalVars>) {
    save_param_change_hist(last_telemetry.timestamp, device_code, unit_id, "CHIL_S_S", last_telemetry.CHIL_S_S, globs);
    save_param_change_hist(last_telemetry.timestamp, device_code, unit_id, "EMSTOP", last_telemetry.EMSTOP, globs);
    save_param_change_hist(last_telemetry.timestamp, device_code, unit_id, "ALM", last_telemetry.ALM, globs);
    save_param_change_hist(last_telemetry.timestamp, device_code, unit_id, "STATUS", last_telemetry.STATUS, globs);
    save_param_change_hist(last_telemetry.timestamp, device_code, unit_id, "CHIL_OCC", last_telemetry.CHIL_OCC, globs);
    save_param_change_hist(last_telemetry.timestamp, device_code, unit_id, "CP_A2", last_telemetry.CP_A2, globs);
    save_param_change_hist(last_telemetry.timestamp, device_code, unit_id, "CP_A1", last_telemetry.CP_A1, globs);
    save_param_change_hist(last_telemetry.timestamp, device_code, unit_id, "CP_B1", last_telemetry.CP_B1, globs);
    save_param_change_hist(last_telemetry.timestamp, device_code, unit_id, "CP_B2", last_telemetry.CP_B2, globs);
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

pub fn calculate_group_averages_hx(grouped_telemetries: &HashMap<NaiveDateTime, Vec<DriChillerCarrierHXTelemetry>>) -> Vec<DriChillerCarrierHXTelemetry> {
    let mut group_averages: Vec<DriChillerCarrierHXTelemetry> = Vec::new();

    for (time_interval, telemetry_array) in grouped_telemetries {
        let mut total_values: HashMap<&str, (f64, usize)> = HashMap::new();

        for telemetry in telemetry_array {
            for (field, value) in &[
                ("CAP_T", telemetry.CAP_T),
                ("DEM_LIM", telemetry.DEM_LIM),
                ("LAG_LIM", telemetry.LAG_LIM),
                ("SP", telemetry.SP),
                ("CTRL_PNT", telemetry.CTRL_PNT),
                ("CAPA_T", telemetry.CAPA_T),
                ("DP_A", telemetry.DP_A),
                ("SP_A", telemetry.SP_A),
                ("SCT_A", telemetry.SCT_A),
                ("SST_A", telemetry.SST_A),
                ("CAPB_T", telemetry.CAPB_T),
                ("DP_B", telemetry.DP_B),
                ("SP_B", telemetry.SP_B),
                ("SCT_B", telemetry.SCT_B),
                ("SST_B", telemetry.SST_B),
                ("COND_LWT", telemetry.COND_LWT),
                ("COND_EWT", telemetry.COND_EWT),
                ("COOL_LWT", telemetry.COOL_LWT),
                ("COOL_EWT", telemetry.COOL_EWT),
                ("CPA1_OP", telemetry.CPA1_OP),
                ("CPA2_OP", telemetry.CPA2_OP),
                ("DOP_A1", telemetry.DOP_A1),
                ("DOP_A2", telemetry.DOP_A2),
                ("CPA1_DGT", telemetry.CPA1_DGT),
                ("CPA2_DGT", telemetry.CPA2_DGT),
                ("EXV_A", telemetry.EXV_A),
                ("HR_CP_A1", telemetry.HR_CP_A1),
                ("HR_CP_A2", telemetry.HR_CP_A2),
                ("CPA1_TMP", telemetry.CPA1_TMP),
                ("CPA2_TMP", telemetry.CPA2_TMP),
                ("CPA1_CUR", telemetry.CPA1_CUR),
                ("CPA2_CUR", telemetry.CPA2_CUR),
                ("CPB1_OP", telemetry.CPB1_OP),
                ("CPB2_OP", telemetry.CPB2_OP),
                ("DOP_B1", telemetry.DOP_B1),
                ("DOP_B2", telemetry.DOP_B2),
                ("CPB1_DGT", telemetry.CPB1_DGT),
                ("CPB2_DGT", telemetry.CPB2_DGT),
                ("EXV_B", telemetry.EXV_B),
                ("HR_CP_B1", telemetry.HR_CP_B1),
                ("HR_CP_B2", telemetry.HR_CP_B2),
                ("CPB1_TMP", telemetry.CPB1_TMP),
                ("CPB2_TMP", telemetry.CPB2_TMP),
                ("CPB1_CUR", telemetry.CPB1_CUR),
                ("CPB2_CUR", telemetry.CPB2_CUR),
                ("COND_SP", telemetry.COND_SP),
            ] {
                if let Some(v) = value {
                    let (sum, count_val) = total_values.entry(field).or_insert((0.0, 0));
                    *sum += v;
                    *count_val += 1;
                }
            }
        }
        let mut averaged_telemetry = DriChillerCarrierHXTelemetry::new(*time_interval);

        for (field, (total, field_count)) in total_values {
            let avg = total / field_count as f64;
            let avg_rounded = (avg * 100.0).round() / 100.0;
            averaged_telemetry.set_field_average(field, avg_rounded);
        }

        group_averages.push(averaged_telemetry);
    }

    group_averages
}

pub fn insert_chiller_hx_parameters(hist: Vec<DriChillerCarrierHXTelemetry>, device_code: &str, unit_id: i32, globs: &Arc<GlobalVars>) {
    for minute_hist in &hist {
        let history = ChillerHXParametersMinutesHist {
            unit_id,
            device_code: device_code.to_string(),
            cap_t: Decimal::from_f64(minute_hist.CAP_T.unwrap_or(0.0)).unwrap(),
            dem_lim:  Decimal::from_f64(minute_hist.DEM_LIM.unwrap_or(0.0)).unwrap(),
            lag_lim:  Decimal::from_f64(minute_hist.LAG_LIM.unwrap_or(0.0)).unwrap(),
            sp: Decimal::from_f64(minute_hist.SP.unwrap_or(0.0)).unwrap(),
            ctrl_pnt: Decimal::from_f64(minute_hist.CTRL_PNT.unwrap_or(0.0)).unwrap(),
            capa_t: Decimal::from_f64( minute_hist.CAPA_T.unwrap_or(0.0)).unwrap(),
            dp_a: Decimal::from_f64(minute_hist.DP_A.unwrap_or(0.0)).unwrap(),
            sp_a: Decimal::from_f64(minute_hist.SP_A.unwrap_or(0.0)).unwrap(),
            sct_a: Decimal::from_f64(minute_hist.SCT_A.unwrap_or(0.0)).unwrap(),
            sst_a:  Decimal::from_f64(minute_hist.SST_A.unwrap_or(0.0)).unwrap(),
            capb_t: Decimal::from_f64(minute_hist.CAPB_T.unwrap_or(0.0)).unwrap(),
            dp_b: Decimal::from_f64(minute_hist.DP_B.unwrap_or(0.0)).unwrap(),
            sp_b: Decimal::from_f64(minute_hist.SP_B.unwrap_or(0.0)).unwrap(),
            sct_b: Decimal::from_f64(minute_hist.SCT_B.unwrap_or(0.0)).unwrap(),
            sst_b: Decimal::from_f64(minute_hist.SST_B.unwrap_or(0.0)).unwrap(),
            cond_lwt: Decimal::from_f64(minute_hist.COND_LWT.unwrap_or(0.0)).unwrap(),
            cond_ewt: Decimal::from_f64(minute_hist.COND_EWT.unwrap_or(0.0)).unwrap(),
            cool_lwt: Decimal::from_f64(minute_hist.COOL_LWT.unwrap_or(0.0)).unwrap(),
            cool_ewt: Decimal::from_f64(minute_hist.COOL_EWT.unwrap_or(0.0)).unwrap(),
            cpa1_op: Decimal::from_f64(minute_hist.CPA1_OP.unwrap_or(0.0)).unwrap(),
            cpa2_op: Decimal::from_f64(minute_hist.CPA2_OP.unwrap_or(0.0)).unwrap(),
            dop_a1: Decimal::from_f64(minute_hist.DOP_A1.unwrap_or(0.0)).unwrap(),
            dop_a2: Decimal::from_f64(minute_hist.DOP_A2.unwrap_or(0.0)).unwrap(),
            cpa1_dgt: Decimal::from_f64(minute_hist.CPA1_DGT.unwrap_or(0.0)).unwrap(),
            cpa2_dgt: Decimal::from_f64(minute_hist.CPA2_DGT.unwrap_or(0.0)).unwrap(),
            exv_a: Decimal::from_f64(minute_hist.EXV_A.unwrap_or(0.0)).unwrap(),
            hr_cp_a1: Decimal::from_f64(minute_hist.HR_CP_A1.unwrap_or(0.0)).unwrap(),
            hr_cp_a2: Decimal::from_f64(minute_hist.HR_CP_A2.unwrap_or(0.0)).unwrap(),
            cpa1_tmp: Decimal::from_f64(minute_hist.CPA1_TMP.unwrap_or(0.0)).unwrap(),
            cpa2_tmp: Decimal::from_f64(minute_hist.CPA2_TMP.unwrap_or(0.0)).unwrap(),
            cpa1_cur: Decimal::from_f64(minute_hist.CPA1_CUR.unwrap_or(0.0)).unwrap(),
            cpa2_cur: Decimal::from_f64(minute_hist.CPA2_CUR.unwrap_or(0.0)).unwrap(),
            cpb1_op: Decimal::from_f64(minute_hist.CPB1_OP.unwrap_or(0.0)).unwrap(),
            cpb2_op: Decimal::from_f64(minute_hist.CPB2_OP.unwrap_or(0.0)).unwrap(),
            dop_b1: Decimal::from_f64(minute_hist.DOP_B1.unwrap_or(0.0)).unwrap(),
            dop_b2: Decimal::from_f64(minute_hist.DOP_B2.unwrap_or(0.0)).unwrap(),
            cpb1_dgt: Decimal::from_f64(minute_hist.CPB1_DGT.unwrap_or(0.0)).unwrap(),
            cpb2_dgt: Decimal::from_f64(minute_hist.CPB2_DGT.unwrap_or(0.0)).unwrap(),
            exv_b: Decimal::from_f64(minute_hist.EXV_B.unwrap_or(0.0)).unwrap(),
            hr_cp_b1: Decimal::from_f64(minute_hist.HR_CP_B1.unwrap_or(0.0)).unwrap(),
            hr_cp_b2: Decimal::from_f64(minute_hist.HR_CP_B2.unwrap_or(0.0)).unwrap(),
            cpb1_tmp: Decimal::from_f64(minute_hist.CPB1_TMP.unwrap_or(0.0)).unwrap(),
            cpb2_tmp: Decimal::from_f64(minute_hist.CPB2_TMP.unwrap_or(0.0)).unwrap(),
            cpb1_cur: Decimal::from_f64(minute_hist.CPB1_CUR.unwrap_or(0.0)).unwrap(),
            cpb2_cur: Decimal::from_f64(minute_hist.CPB2_CUR.unwrap_or(0.0)).unwrap(),
            cond_sp: Decimal::from_f64(minute_hist.COND_SP.unwrap_or(0.0)).unwrap(),
            record_date: minute_hist.timestamp,
        };

        insert_chiller_hx_parameters_hist(history, globs);
    }
}

pub async fn process_chiller_hx_devices(unit_id: i32, chiller_devices: &Option<Vec<DriDevice>>, day: &str, check_minutes_offline: Option<i32>, globs: &Arc<GlobalVars>) {
    if let Some(devices) = chiller_devices {
        process_chiller_hx_devices_by_script(unit_id, day, &devices, check_minutes_offline, globs).await;
    }
}

pub fn verify_hours_online_hx(chiller_hist: Vec<DriChillerCarrierHXTelemetry>, client_minutes_to_check_offline: Option<i32>, dri_interval: Option<isize>, start_date: NaiveDateTime, device_code: &str, unit_id: i32, day: &str, globs: &Arc<GlobalVars>) {
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
