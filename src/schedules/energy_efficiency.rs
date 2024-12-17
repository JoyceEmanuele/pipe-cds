use std::{sync::{Arc, Mutex}};

use chrono::NaiveDate;
use rust_decimal::{prelude::FromPrimitive, Decimal};

use crate::{db::entities::{energy_efficiency_hour_hist::insert_data_energy_efficiency_hour, machines::{get_machine, insert_data_machine, update_machine}}, models::{database_models::{energy_efficiency_hist::EnergyEfficiencyHist, energy_efficiency_hour_hist::EnergyEfficiencyHourHist, machines::Machines}, external_models::device::{DacDevice, DutDevice}}, schema::chiller_parameters_changes_hist::record_date, GlobalVars};

use super::devices::{process_dacs_devices, process_duts_devices};

pub async fn process_energy_efficiency_dacs(unit_id: i32, day: &str, dacs_devices: &Option<Vec<DacDevice>>, client_minutes_to_check_offline: Option<i32>, globs: &Arc<GlobalVars>) {
    if let Some(devices) = dacs_devices {
        process_dacs_devices(unit_id, day, devices, client_minutes_to_check_offline, globs).await;
    }
}

pub async fn process_energy_efficiency_duts(unit_id: i32, day: &str, duts_devices: &Option<Vec<DutDevice>>, client_minutes_to_check_offline: Option<i32>, globs: &Arc<GlobalVars>) {
    if let Some(devices) = duts_devices {
        process_duts_devices(unit_id, day, devices, client_minutes_to_check_offline, globs).await;
    }
}

pub fn insert_energy_efficiency_hour_history(
    device_code: &str,
    machine_id: i32,
    capacity_power: Option<Decimal>,
    day: &str,
    vec_utilization_time: Vec<i32>,
    globs: &Arc<GlobalVars>
) {
    let machine_kw = capacity_power.unwrap_or(Decimal::new(0, 0));
    for hour in 0..=23 {
        let hour_value = vec_utilization_time[hour];
        let hour_u32 = hour as u32;

        let day_parsed = NaiveDate::parse_from_str(day, "%Y-%m-%d").unwrap();
        let record_date_aux = day_parsed.and_hms_opt(hour_u32, 0, 0).unwrap();
        let utilization_time = Decimal::from_i32(hour_value).unwrap_or(Decimal::new(0, 0))/Decimal::new(3600, 0).round_dp(3);
        let consumption = (machine_kw * utilization_time).round_dp(3);

        let history = EnergyEfficiencyHourHist {
            machine_id,
            device_code: device_code.to_string(),
            consumption,
            utilization_time,
            record_date: record_date_aux,
        };

        insert_data_energy_efficiency_hour(history, globs);
    }
}


pub fn verify_insert_update_machine(unit_id: i32, machine_id: i32, machine_name: &str, device_code_autom: &str, globs: &Arc<GlobalVars>) -> Result<i32, String> {
    let actual_machine_info = get_machine(machine_id, globs);

    let machine = match actual_machine_info {
        Ok(machine_tuple) => machine_tuple,
        Err(err) => return Err(format!("Erro ao verificar Máquina, {}", err))
    };

    if let Some(machine_tuple) = machine {
        let device_code_autom_matches = match &machine_tuple.2 {
            Some(code) => code == device_code_autom,
            None => device_code_autom.is_empty(),
        };

        if machine_tuple.1 != machine_name || !device_code_autom_matches {
            update_machine(machine_tuple.0, machine_name, device_code_autom, globs);
        }
        Ok(machine_tuple.0)
    } else {
        let machine = Machines {
            id: None,
            unit_id,
            machine_name: machine_name.to_string(),
            reference_id: machine_id,
            device_code_autom: if device_code_autom.is_empty() {
                None
            } else {
                Some(device_code_autom.to_string())
            },
        };

       let inserted_id = insert_data_machine(machine, globs).unwrap();
       Ok(inserted_id)
    }
}
