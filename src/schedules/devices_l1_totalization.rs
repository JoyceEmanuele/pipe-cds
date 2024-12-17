use std::{sync::{Arc, Mutex}};

use chrono::NaiveDate;
use rust_decimal::{prelude::FromPrimitive, Decimal};

use crate::{db::entities::{devices_l1_totalization_hist::insert_data_device_l1_totalization_hist, assets::{insert_data_asset, update_asset, get_asset}}, models::{database_models::{devices_l1_totalization_hist::DevicesL1TotalizationHist, assets::Assets}, external_models::device::{DacDevice, DutDevice}},GlobalVars};

use super::devices::{process_dacs_devices, process_duts_devices};

pub async fn process_l1_totalization_dacs(unit_id: i32, day: &str, dacs_devices: &Option<Vec<DacDevice>>, client_minutes_to_check_offline: Option<i32>,  globs: &Arc<GlobalVars>) {
    if let Some(devices) = dacs_devices {
        process_dacs_devices(unit_id, day, devices, client_minutes_to_check_offline, globs).await;
    }
}

pub async fn process_l1_totalization_duts(unit_id: i32, day: &str, duts_devices: &Option<Vec<DutDevice>>, client_minutes_to_check_offline: Option<i32>, globs: &Arc<GlobalVars>) {
    if let Some(devices) = duts_devices {
        process_duts_devices(unit_id, day, devices, client_minutes_to_check_offline, globs).await;
    }
}

pub fn insert_device_l1_totalization(
    asset_reference_id: Option<i32>,
    machine_reference_id: i32,
    device_code: &str,
    seconds_on: i32,
    seconds_off: i32,
    seconds_on_outside_programming: i32,
    seconds_must_be_off: i32,
    percentage_on_outside_programming: Decimal,
    programming: &str,
    day: &str,
    globs: &Arc<GlobalVars>
) {
    let percentage_on_outside_programming_formatted = percentage_on_outside_programming.round_dp(2);

    let history = DevicesL1TotalizationHist {
        asset_reference_id,
        machine_reference_id,
        device_code: device_code.to_string(),
        record_date: NaiveDate::parse_from_str(day, "%Y-%m-%d")
        .unwrap_or_default(),
        seconds_on,
        seconds_off,
        seconds_on_outside_programming,
        seconds_must_be_off,
        percentage_on_outside_programming: percentage_on_outside_programming_formatted,
        programming: programming.to_string(),
    };

    insert_data_device_l1_totalization_hist(history, globs);
}

pub fn verify_insert_update_asset(unit_id: i32, asset_id: i32, device_code: &str, asset_name: &str, machine_reference_id: i32, globs: &Arc<GlobalVars>) -> Result<i32, String> {
    let actual_asset_info = get_asset(asset_id, globs);

    let asset = match actual_asset_info {
        Ok(asset_tuple) => asset_tuple,
        Err(err) => return Err(format!("Error to check Asset, {}", err))
    };

    if let Some(asset_tuple) = asset {
        if asset_tuple.1 != asset_name || asset_tuple.2 != device_code || asset_tuple.3 != machine_reference_id {
            update_asset(asset_tuple.0, asset_name, device_code, machine_reference_id, globs);
        }
        Ok(asset_tuple.0)
    } else {
        let asset = Assets {
            id: None,
            unit_id,
            asset_name: asset_name.to_string(),
            device_code: device_code.to_string(),
            machine_reference_id: machine_reference_id,
            reference_id: asset_id,
        };

       let inserted_id = insert_data_asset(asset, globs).unwrap();
       Ok(inserted_id)
    }
}
