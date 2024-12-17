use std::sync::{Arc, Mutex};

use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::{db::entities::disponibility_hist::{self, insert_data_disponibility_hist}, models::external_models::{device::{DacDevice, DalDevice, DamDevice, DmtDevice, DriDevice, DutDevice}}, GlobalVars};
use crate::models::database_models::disponibility_hist::DisponibilityHist;

use super::devices::{process_dacs_devices, process_dals_devices, process_dams_devices, process_dmts_devices, process_dris_devices, process_duts_devices};


pub async fn process_disponibility_duts_devices(unit_id: i32, day: &str, duts_devices: &Option<Vec<DutDevice>>, client_minutes_to_check_offline: Option<i32>, globs: &Arc<GlobalVars>) {
    if let Some(devices) = duts_devices {
        process_duts_devices(unit_id, day, devices, client_minutes_to_check_offline, globs).await;
    }
}

pub async fn process_disponibility_dacs_devices(unit_id: i32, day: &str, dacs_devices: &Option<Vec<DacDevice>>, client_minutes_to_check_offline: Option<i32>, globs: &Arc<GlobalVars>) {
    if let Some(devices) = dacs_devices {
        process_dacs_devices(unit_id, day, devices, client_minutes_to_check_offline, globs).await;
    }
}

pub async fn process_disponibility_dris_devices(unit_id: i32, day: &str, dris_devices: &Option<Vec<DriDevice>>, client_minutes_to_check_offline: Option<i32>, globs: &Arc<GlobalVars>) {
    if let Some(devices) = dris_devices {
        process_dris_devices(unit_id, day, devices, client_minutes_to_check_offline, globs).await;
    }
}

pub async fn process_disponibility_dmts_devices(unit_id: i32, day: &str, dmts_devices: &Option<Vec<DmtDevice>>, client_minutes_to_check_offline: Option<i32>, globs: &Arc<GlobalVars>) {
    if let Some(devices) = dmts_devices {
        process_dmts_devices(unit_id, day, devices, client_minutes_to_check_offline, globs).await;
    }
}

pub async fn process_disponibility_dals_devices(unit_id: i32, day: &str, dals_devices: &Option<Vec<DalDevice>>, client_minutes_to_check_offline: Option<i32>, globs: &Arc<GlobalVars>) {
    if let Some(devices) = dals_devices {
        process_dals_devices(unit_id, day, devices, client_minutes_to_check_offline, globs).await;
    }
}

pub async fn process_disponibility_dams_devices(unit_id: i32, day: &str, dams_devices: &Option<Vec<DamDevice>>, client_minutes_to_check_offline: Option<i32>, globs: &Arc<GlobalVars>) {
    if let Some(devices) = dams_devices {
        process_dams_devices(unit_id, day, devices, client_minutes_to_check_offline, globs).await;
    }
}

pub fn insert_disponibility_hist(
    unit_id: i32,
    disponibility: Decimal,
    day: &str,
    globs: &Arc<GlobalVars>
) {
    let history = DisponibilityHist {
        unit_id,
        record_date: NaiveDate::parse_from_str(day, "%Y-%m-%d")
            .unwrap_or_default(),
        disponibility: Decimal::from_str_exact(&format!("{:.4}", &disponibility)).unwrap().round_dp(1),
    };

    insert_data_disponibility_hist(history, globs);
}