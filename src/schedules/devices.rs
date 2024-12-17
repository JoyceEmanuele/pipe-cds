use std::sync::{Arc, Mutex};
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use tokio::sync::Semaphore;
use rust_decimal::prelude::*;
use crate::compression::common_func::{consumption_by_hour, calculate_l1_states, concatenate_intervals};
use crate::external_api::api_server::ApiServer;
use crate::schedules::waters::forecast_without_consumption_by_day;
use crate::telemetry_payloads::dri_telemetry::DriChillerCarrierXAHvarTelemetry;
use super::chiller::chiller_hx_parameters::verify_hours_online_hx;
use super::chiller::chiller_xa_parameters::{calculate_group_averages_xa_hvar, convert_to_simple_xa, convert_to_simple_xa_hvar, group_telemetries_by_10_minutes_xa_hvar, insert_chiller_xa_hvar_parameters, verify_hours_online_xa};
use super::device_disponibility::insert_device_disponibility_hist;
use super::last_device_telemetry_time::{insert_last_device_telemetry, very_last_device_telemetry_dac};
use super::waters::{compile_dma_data, compile_laager_data, insert_data_dma_per_hour, insert_data_laager_per_hour, normalize_laager_consumption};
use super::devices_l1_totalization::{insert_device_l1_totalization, verify_insert_update_asset};

use crate::{app_history::{compiler_queues::{task_queue_manager, CompilationRequest}, dac_hist::{parse_parameters_dac, DacHist}, dal_hist::{parse_parameters_dal, DalHist}, dam_hist::{parse_parameters_dam, DamHist}, dma_hist::{parse_parameters, DmaCompiledData}, dmt_hist::{parse_parameters_dmt, DmtHist}, dri_hist::{DriHist, DriHistParams}, dut_hist::{parse_parameters_dut, DutHist}}, external_api::api_laager::LaagerApi, models::external_models::{device::{DacDevice, DalDevice, DamDevice, DmaDevice, DmtDevice, DriDevice, DutDevice, LaagerDevice, WaterConsumptionHistory}, unit}, schedules::scheduler::write_to_log_file_thread, telemetry_payloads::dri_telemetry::{DriChillerCarrierHXTelemetry, DriChillerCarrierXATelemetry}, GlobalVars};
use super::chiller::{chiller_hx_parameters::{calculate_group_averages_hx, group_telemetries_by_10_minutes_hx, insert_chiller_hx_parameters}, chiller_xa_parameters::{calculate_group_averages_xa, group_telemetries_by_10_minutes_xa, insert_chiller_xa_parameters}};
use super::energy_efficiency::{insert_energy_efficiency_hour_history, verify_insert_update_machine};

pub async fn process_duts_devices(unit_id: i32, day: &str, devices: &Vec<DutDevice>, client_minutes_to_check_offline: Option<i32>, globs: &Arc<GlobalVars>) {
    for dut_device in devices {
        let params = match parse_parameters_dut(&dut_device.device_code, dut_device.temperature_offset, day, client_minutes_to_check_offline) {
            Ok(params) => params,
            Err(err) => {
                insert_device_disponibility_hist(unit_id, Decimal::new(0, 0), day, &dut_device.device_code, globs);
                continue;
            }
        };

        let response = match task_queue_manager(CompilationRequest::CompDut(params), &globs).await {
            Ok(response) => response,
            Err(err) => {
                insert_device_disponibility_hist(unit_id, Decimal::new(0, 0), day, &dut_device.device_code, globs);
                eprintln!("Erro ao obter response DUT: {}, {}", dut_device.device_code, err);
                continue;
            }
        };

        let response_data = match serde_json::from_str::<DutHist>(&response) {
            Ok(resp) => resp,
            Err(err) => {
                insert_device_disponibility_hist(unit_id, Decimal::new(0, 0), day, &dut_device.device_code, globs);
                continue;
            },
        };

        if let (Some(machine_id_value), Some(true)) = (dut_device.machine_id, dut_device.has_energy_efficiency) {
            let device_code_autom_ref = dut_device.device_code_autom.as_deref().unwrap_or("");
    
            if let Ok(machine_id) = verify_insert_update_machine(unit_id, machine_id_value, &dut_device.machine_name.clone().unwrap(), device_code_autom_ref, &globs) {
                let vec_utilization_time: Vec<i32> = consumption_by_hour(&response_data.lcmp);
                insert_energy_efficiency_hour_history(&dut_device.device_code, machine_id, dut_device.machine_kw, day, vec_utilization_time, &globs);
            } else {
                continue;
            }
        }


        if let Some(ref intervals) = dut_device.machine_autom_intervals {
            if !intervals.is_empty() {
                let device_code_autom_ref = dut_device.device_code_autom.as_deref().unwrap_or("");
                if let Ok(machine_id) = verify_insert_update_machine(unit_id, dut_device.machine_id.unwrap(), &dut_device.machine_name.clone().unwrap(), device_code_autom_ref, &globs) {
                    if let Some(asset_id_valie) = dut_device.asset_id {
                        if let Ok(asset_id) = verify_insert_update_asset(unit_id, dut_device.asset_id.unwrap(), &dut_device.device_code, &dut_device.asset_name.clone().unwrap(), dut_device.machine_id.unwrap(), &globs) {
                            // do nothing
                        }
                        else {
                            continue;
                        }
                    }
                    let (total_on, total_off, total_on_outside_programming, seconds_must_be_off, percentual_outside_programming) = calculate_l1_states(&response_data.lcmp, intervals.clone());
                    let programming = concatenate_intervals(intervals.clone());
                    let percentual_outside_programming_converted = Decimal::from_f64(percentual_outside_programming).unwrap_or(Decimal::new(0, 0));
                    insert_device_l1_totalization(dut_device.asset_id, dut_device.machine_id.unwrap(), &dut_device.device_code, total_on, total_off, total_on_outside_programming, seconds_must_be_off, percentual_outside_programming_converted, &programming, day, &globs);       
                } else {
                    continue;
                }              
            }
        }

        insert_device_disponibility_hist(unit_id, response_data.hours_on, day, &dut_device.device_code, globs);
    }
}

pub async fn process_dacs_devices(unit_id: i32, day: &str, devices: &Vec<DacDevice>, client_minutes_to_check_offline: Option<i32>, globs: &Arc<GlobalVars>) {
    let semaphore = Arc::new(Semaphore::new(5));

    let tasks = devices.iter().map(|dac_device| {
        let semaphore = Arc::clone(&semaphore);
        let day = day.to_string();
        let globs = globs.clone();

        async move {
            let permit = semaphore.acquire_owned().await.unwrap();

            match process_single_dac_device(unit_id, &day, dac_device, client_minutes_to_check_offline, &globs).await {
                Ok(_) => {}
                Err(err) => {
                    // eprintln!("Erro ao processar DAC {}, {}", dac_device.device_code, err)
                }
            }

            drop(permit);
        }
    });

    futures::future::join_all(tasks).await;
}

pub async fn process_single_dac_device(unit_id: i32, day: &str, dac_device: &DacDevice, client_minutes_to_check_offline: Option<i32>, globs: &Arc<GlobalVars>) -> Result<(), String> {
    let response_data = process_dac_common(unit_id, day, dac_device, client_minutes_to_check_offline, globs).await?;

    // reprocess if the firmware version is equal to or greater than v4.x.x
    if dac_device.v_major.unwrap_or(0) >= 4 {
        very_last_device_telemetry_dac(day, unit_id, &dac_device, client_minutes_to_check_offline, globs).await;
        insert_last_device_telemetry(&response_data.last_telemetry_time, &dac_device.device_code, globs);
    }


    Ok(())
}

pub async fn process_dac_common(
    unit_id: i32,
    day: &str,
    dac_device: &DacDevice,
    client_minutes_to_check_offline: Option<i32>,
    globs: &Arc<GlobalVars>,
) -> Result<DacHist, String> {
    let params = match parse_parameters_dac(dac_device, day, client_minutes_to_check_offline) {
        Ok(params) => params,
        Err(err) => {
            insert_device_disponibility_hist(
                unit_id,
                Decimal::new(0, 0),
                day,
                &dac_device.device_code,
                globs,
            );
            return Err(format!(
                "Erro ao obter parâmetros DAC: {}, {}",
                dac_device.device_code, err
            ));
        }
    };

    let response = match task_queue_manager(CompilationRequest::CompDac(params), globs).await {
        Ok(response) => response,
        Err(err) => {
            insert_device_disponibility_hist(
                unit_id,
                Decimal::new(0, 0),
                day,
                &dac_device.device_code,
                globs,
            );
            return Err(format!(
                "Erro ao obter response DAC: {}, {}",
                dac_device.device_code, err
            ));
        }
    };

    let response_data = match serde_json::from_str::<DacHist>(&response) {
        Ok(resp) => resp,
        Err(err) => {
            insert_device_disponibility_hist(
                unit_id,
                Decimal::new(0, 0),
                day,
                &dac_device.device_code,
                globs,
            );
            return Err(format!(
                "Erro ao desserealizar JSON: {} {} {}",
                dac_device.device_code, err, response
            ));
        }
    };

    if let Some(machine_id_value) = dac_device.machine_id {
        let device_code_autom_ref = dac_device.device_code_autom.as_deref().unwrap_or("");;
        if let Ok(machine_id) = verify_insert_update_machine(unit_id, machine_id_value, &dac_device.machine_name.clone().unwrap(), device_code_autom_ref, globs) {
            let vec_utilization_time: Vec<i32> = consumption_by_hour(&response_data.lcmp);
            insert_energy_efficiency_hour_history(&dac_device.device_code, machine_id, dac_device.machine_kw, day, vec_utilization_time, &globs);
        } else {
            return Err("Erro ao inserir dados".to_string());
        }
    }

    if let Some(ref intervals) = dac_device.machine_autom_intervals {
        if !intervals.is_empty() {
            if let Ok(asset_id) = verify_insert_update_asset(unit_id, dac_device.asset_id.unwrap(), &dac_device.device_code, &dac_device.asset_name.clone().unwrap(), dac_device.machine_id.unwrap(), &globs) {
                let (total_on, total_off, total_on_outside_programming, seconds_must_be_off, percentual_outside_programming) = calculate_l1_states(&response_data.lcmp, intervals.clone());
                let programming = concatenate_intervals(intervals.clone());
                let percentual_outside_programming_converted = Decimal::from_f64(percentual_outside_programming).unwrap_or(Decimal::new(0, 0));
                insert_device_l1_totalization(dac_device.asset_id, dac_device.machine_id.unwrap(), &dac_device.device_code, total_on, total_off, total_on_outside_programming, seconds_must_be_off, percentual_outside_programming_converted, &programming, day, &globs);
            } else {
                return Err("Erro ao inserir dados de ativos".to_string());
            }
        }
    }

    insert_device_disponibility_hist(unit_id, response_data.hours_dev_on, day, &dac_device.device_code, globs);

    Ok(response_data)
}

pub async fn process_dris_devices(unit_id: i32, day: &str, devices: &Vec<DriDevice>, client_minutes_to_check_offline: Option<i32>, globs: &Arc<GlobalVars>) {
    for dri_device in devices {        
        let params = match DriHistParams::parse_parameters_dri(dri_device, day, client_minutes_to_check_offline) {
            Ok(params) => params,
            Err(err) => {
                insert_device_disponibility_hist(unit_id, Decimal::new(0,0), day, &dri_device.dev_id, globs);
                continue;
            }
        };


        let response = match task_queue_manager(CompilationRequest::CompDri(params.clone()), globs).await {
            Ok(response) => response,
            Err(err) => {
                insert_device_disponibility_hist(unit_id, Decimal::new(0,0), day, &dri_device.dev_id, globs);
                eprintln!("Erro ao obter response DRI: {}, {}", dri_device.dev_id, err);
                continue;
            }
        };

        if params.dri_type == "CHILLER_CARRIER_XA" {
            let response_data = match serde_json::from_str::<Vec<DriChillerCarrierXATelemetry>>(&response) {
                Ok(resp) => resp,
                Err(err) => {
                    insert_device_disponibility_hist(unit_id, Decimal::new(0,0), day, &dri_device.dev_id, globs);
                    continue;
                },
            };
            let grouped_telemetries = group_telemetries_by_10_minutes_xa(&dri_device.dev_id, unit_id, response_data.clone(), globs);
            let grouped_averages = calculate_group_averages_xa(&grouped_telemetries);
            insert_chiller_xa_parameters(grouped_averages, &dri_device.dev_id, unit_id, globs);

            let simple_telemetry = convert_to_simple_xa(response_data.clone());
            verify_hours_online_xa(simple_telemetry, client_minutes_to_check_offline, dri_device.dri_interval, params.day.and_hms(0, 0, 0),&dri_device.dev_id, unit_id, day, globs);
        } else if params.dri_type == "CHILLER_CARRIER_HX" {
            let response_data = match serde_json::from_str::<Vec<DriChillerCarrierHXTelemetry>>(&response) {
                Ok(resp) => resp,
                Err(err) => {
                    insert_device_disponibility_hist(unit_id, Decimal::new(0,0), day, &dri_device.dev_id, globs);
                    continue;
                },
            };
            let grouped_telemetries = group_telemetries_by_10_minutes_hx(&dri_device.dev_id, unit_id, response_data.clone(), globs);
            let grouped_averages = calculate_group_averages_hx(&grouped_telemetries);
            insert_chiller_hx_parameters(grouped_averages, &dri_device.dev_id, unit_id, globs);
            
            verify_hours_online_hx(response_data.clone(), client_minutes_to_check_offline, dri_device.dri_interval, params.day.and_hms(0, 0, 0),&dri_device.dev_id, unit_id, day, globs);
        } else if params.dri_type == "CHILLER_CARRIER_XA_HVAR" {
            let response_data = match serde_json::from_str::<Vec<DriChillerCarrierXAHvarTelemetry>>(&response) {
                Ok(resp) => resp,
                Err(err) => {
                    insert_device_disponibility_hist(unit_id, Decimal::new(0,0), day, &dri_device.dev_id, globs);
                    continue;
                },
            };
            let grouped_telemetries = group_telemetries_by_10_minutes_xa_hvar(&dri_device.dev_id, unit_id, response_data.clone(), globs);
            let grouped_averages = calculate_group_averages_xa_hvar(&grouped_telemetries);
            insert_chiller_xa_hvar_parameters(grouped_averages, &dri_device.dev_id, unit_id, globs);

            let simple_telemetry = convert_to_simple_xa_hvar(response_data.clone());
            verify_hours_online_xa(simple_telemetry, client_minutes_to_check_offline, dri_device.dri_interval, params.day.and_hms(0, 0, 0),&dri_device.dev_id, unit_id, day, globs);
        }
        else {
            let response_data = match serde_json::from_str::<DriHist>(&response) {
                Ok(resp) => resp,
                Err(err) => {
                    insert_device_disponibility_hist(unit_id, Decimal::new(0,0), day, &dri_device.dev_id, globs);
                    continue;
                },
            };

            insert_device_disponibility_hist(unit_id, response_data.hours_online, day, &dri_device.dev_id, globs);
        } 
    }
}

pub async fn process_dmts_devices(unit_id: i32, day: &str, devices: &Vec<DmtDevice>, client_minutes_to_check_offline: Option<i32>, globs: &Arc<GlobalVars>) {
    for dmt_device in devices {        
        let params = match parse_parameters_dmt(day, &dmt_device.device_code, client_minutes_to_check_offline) {
            Ok(params) => params,
            Err(err) => {
                insert_device_disponibility_hist(unit_id, Decimal::new(0,0), day, &dmt_device.device_code, globs);
                continue;
            }
        };

        let response = match task_queue_manager(CompilationRequest::CompDmt(params), globs).await {
            Ok(response) => response,
            Err(err) => {
                eprintln!("Erro ao obter response DMT: {}, {}", dmt_device.device_code, err);
                insert_device_disponibility_hist(unit_id, Decimal::new(0,0), day, &dmt_device.device_code, globs);
                continue;
            }
        };

        let response_data = match serde_json::from_str::<DmtHist>(&response) {
            Ok(resp) => resp,
            Err(err) => {
                insert_device_disponibility_hist(unit_id, Decimal::new(0,0), day, &dmt_device.device_code, globs);
                continue;
            },
        };

        insert_device_disponibility_hist(unit_id, response_data.hours_online, day, &dmt_device.device_code, globs);
    }
}

pub async fn process_dals_devices(unit_id: i32, day: &str, devices: &Vec<DalDevice>, client_minutes_to_check_offline: Option<i32>, globs: &Arc<GlobalVars>) {
    for dal_device in devices {        
        let params = match parse_parameters_dal(day, &dal_device.device_code, client_minutes_to_check_offline) {
            Ok(params) => params,
            Err(err) => {
                insert_device_disponibility_hist(unit_id, Decimal::new(0,0), day, &dal_device.device_code, globs);
                continue;
            }
        };

        let response = match task_queue_manager(CompilationRequest::CompDal(params), globs).await {
            Ok(response) => response,
            Err(err) => {
                eprintln!("Erro ao obter response DAL: {}, {}", dal_device.device_code, err);
                insert_device_disponibility_hist(unit_id, Decimal::new(0,0), day, &dal_device.device_code, globs);
                continue;
            }
        };

        let response_data = match serde_json::from_str::<DalHist>(&response) {
            Ok(resp) => resp,
            Err(err) => {
                insert_device_disponibility_hist(unit_id, Decimal::new(0,0), day, &dal_device.device_code, globs);

                continue;
            },
        };


        insert_device_disponibility_hist(unit_id, response_data.hours_online, day, &dal_device.device_code, globs);
    }
}

pub async fn process_dams_devices(unit_id: i32, day: &str, devices: &Vec<DamDevice>, client_minutes_to_check_offline: Option<i32>, globs: &Arc<GlobalVars>) {
    for dam_device in devices {        
        let params = match parse_parameters_dam(day, &dam_device.device_code, client_minutes_to_check_offline) {
            Ok(params) => params,
            Err(err) => {
                insert_device_disponibility_hist(unit_id, Decimal::new(0, 0), day, &dam_device.device_code, globs);
                continue;
            }
        };

        let response = match task_queue_manager(CompilationRequest::CompDam(params), globs).await {
            Ok(response) => response,
            Err(err) => {
                insert_device_disponibility_hist(unit_id, Decimal::new(0, 0), day, &dam_device.device_code, globs);
                eprintln!("Erro ao obter response DAM: {}, {}", dam_device.device_code, err);
                continue;
            }
        };

        let response_data = match serde_json::from_str::<DamHist>(&response) {
            Ok(resp) => resp,
            Err(err) => {
                insert_device_disponibility_hist(unit_id, Decimal::new(0, 0), day, &dam_device.device_code, globs);
                continue;
            },
        };

        insert_device_disponibility_hist(unit_id, response_data.hours_online, day, &dam_device.device_code, globs);
    }
}

pub async fn process_chiller_hx_devices_by_script(unit_id: i32, day: &str, devices: &Vec<DriDevice>, check_minutes_offline: Option<i32>, globs: &Arc<GlobalVars>) {
    for dri_device in devices {        
        let params = match DriHistParams::parse_parameters_dri(dri_device, day, check_minutes_offline) {
            Ok(params) => params,
            Err(err) => {
                insert_device_disponibility_hist(unit_id, Decimal::new(0,0), day, &dri_device.dev_id, globs);
                continue;
            }
        };

        if params.dri_type != "CHILLER_CARRIER_HX" { continue; } ;

        let response = match task_queue_manager(CompilationRequest::CompDri(params.clone()), globs).await {
            Ok(response) => response,
            Err(err) => {
                insert_device_disponibility_hist(unit_id, Decimal::new(0,0), day, &dri_device.dev_id, globs);
                continue;
            }
        };

        let response_data = match serde_json::from_str::<Vec<DriChillerCarrierHXTelemetry>>(&response) {
            Ok(resp) => resp,
            Err(err) => {
                insert_device_disponibility_hist(unit_id, Decimal::new(0,0), day, &dri_device.dev_id, globs);
                continue;
            },
        };

        let grouped_telemetries = group_telemetries_by_10_minutes_hx(&dri_device.dev_id, unit_id, response_data.clone(), globs);
        let grouped_averages = calculate_group_averages_hx(&grouped_telemetries);
        insert_chiller_hx_parameters(grouped_averages, &dri_device.dev_id, unit_id, globs);

        verify_hours_online_hx(response_data.clone(), check_minutes_offline, dri_device.dri_interval, params.day.and_hms(0, 0, 0),&dri_device.dev_id, unit_id, day, globs);
    }
}

pub async fn process_chiller_xa_devices_by_script(unit_id: i32, day: &str, devices: &Vec<DriDevice>, client_minutes_to_check_offline: Option<i32>, globs: &Arc<GlobalVars>) {
    for dri_device in devices {        
        let params = match DriHistParams::parse_parameters_dri(dri_device, day, client_minutes_to_check_offline) {
            Ok(params) => params,
            Err(err) => {
                insert_device_disponibility_hist(unit_id, Decimal::new(0,0), day, &dri_device.dev_id, globs);
                continue;
            }
        };

        if params.dri_type != "CHILLER_CARRIER_XA" { continue; } ;

        let response = match task_queue_manager(CompilationRequest::CompDri(params.clone()), globs).await {
            Ok(response) => response,
            Err(err) => {
                insert_device_disponibility_hist(unit_id, Decimal::new(0,0), day, &dri_device.dev_id, globs);
                continue;
            }
        };

        let response_data = match serde_json::from_str::<Vec<DriChillerCarrierXATelemetry>>(&response) {
            Ok(resp) => resp,
            Err(err) => {
                insert_device_disponibility_hist(unit_id, Decimal::new(0,0), day, &dri_device.dev_id, globs);
                continue;
            },
        };

        let grouped_telemetries = group_telemetries_by_10_minutes_xa(&dri_device.dev_id, unit_id, response_data.clone(), globs);

        let grouped_averages = calculate_group_averages_xa(&grouped_telemetries);

        insert_chiller_xa_parameters(grouped_averages, &dri_device.dev_id, unit_id, globs);

        let simple_telemetry = convert_to_simple_xa(response_data.clone());
        verify_hours_online_xa(simple_telemetry, client_minutes_to_check_offline, dri_device.dri_interval, params.day.and_hms(0, 0, 0),&dri_device.dev_id, unit_id, day, globs);
    }
}

pub async fn process_chiller_xa_hvar_devices_by_script(unit_id: i32, day: &str, devices: &Vec<DriDevice>, client_minutes_to_check_offline: Option<i32>, globs: &Arc<GlobalVars>) {
    for dri_device in devices {        
        let params = match DriHistParams::parse_parameters_dri(dri_device, day, client_minutes_to_check_offline) {
            Ok(params) => params,
            Err(_) => {
                insert_device_disponibility_hist(unit_id, Decimal::new(0,0), day, &dri_device.dev_id, globs);
                continue;
            }
        };

        if params.dri_type != "CHILLER_CARRIER_XA_HVAR" { continue; } ;

        let response = match task_queue_manager(CompilationRequest::CompDri(params.clone()), globs).await {
            Ok(response) => response,
            Err(_) => {
                insert_device_disponibility_hist(unit_id, Decimal::new(0,0), day, &dri_device.dev_id, globs);
                continue;
            }
        };

        let response_data = match serde_json::from_str::<Vec<DriChillerCarrierXAHvarTelemetry>>(&response) {
            Ok(resp) => resp,
            Err(_) => {
                insert_device_disponibility_hist(unit_id, Decimal::new(0,0), day, &dri_device.dev_id, globs);
                continue;
            },
        };

        let grouped_telemetries = group_telemetries_by_10_minutes_xa_hvar(&dri_device.dev_id, unit_id, response_data.clone(), globs);

        let grouped_averages = calculate_group_averages_xa_hvar(&grouped_telemetries);

        insert_chiller_xa_hvar_parameters(grouped_averages, &dri_device.dev_id, unit_id, globs);
        let simple_telemetry = convert_to_simple_xa_hvar(response_data.clone());

        verify_hours_online_xa(simple_telemetry, client_minutes_to_check_offline, dri_device.dri_interval, params.day.and_hms(0, 0, 0),&dri_device.dev_id, unit_id, day, globs);
    }
}

pub async fn process_dmas_devices_per_hour(unit_id: i32, day: &str, dma_device: &DmaDevice, client_minutes_to_check_offline: Option<i32>, globs: &Arc<GlobalVars>) {
    let params = match parse_parameters(&dma_device.device_code, day, client_minutes_to_check_offline) {
        Ok(params) => params,
        Err(err) => {
            insert_device_disponibility_hist(unit_id, Decimal::new(0,0), day, &dma_device.device_code, globs);
            forecast_without_consumption_by_day(dma_device.installation_date.clone(), day, unit_id, globs);
            return;
        }
    };

    let response = match task_queue_manager(CompilationRequest::CompDma(params), globs).await {
        Ok(response) => response,
        Err(err) => {
            insert_device_disponibility_hist(unit_id, Decimal::new(0,0), day, &dma_device.device_code, globs);
            forecast_without_consumption_by_day(dma_device.installation_date.clone(), day, unit_id, globs);
            eprintln!("Erro ao obter response DMA: {}, {}", dma_device.device_code, err);
            return;
        }
    };

    let dma_response_result: Result<DmaCompiledData, _> = serde_json::from_str(&response);

    match dma_response_result {
        Ok(dma_response) => {
            let compiled_dma_data = compile_dma_data(&dma_response, day);
            if let Some(liters_per_pulse) = dma_device.liters_per_pulse {
                insert_data_dma_per_hour(
                    &dma_device.device_code,
                    liters_per_pulse,
                    "Diel",
                    unit_id,
                    &compiled_dma_data,
                    dma_device.installation_date.as_deref(),
                    globs).await;
            }
            insert_device_disponibility_hist(unit_id, Decimal::from_f64_retain(dma_response.hours_online).unwrap_or(Decimal::new(0,0)), day, &dma_device.device_code, globs);
        }
        Err(err) => {
            insert_device_disponibility_hist(unit_id, Decimal::new(0,0), day, &dma_device.device_code, globs);
            forecast_without_consumption_by_day(dma_device.installation_date.clone(), day, unit_id, globs);
        }
    }
}

pub async fn process_laager_devices_per_hour(unit_id: i32, day: &str, laager_device: &LaagerDevice, globs: &Arc<GlobalVars>) {
    let rf_device_id = match LaagerApi::verify_laager_meter(&laager_device.laager_code, globs).await {
        Ok(rf_device_id) => rf_device_id,
        Err(err) => { 
            forecast_without_consumption_by_day(laager_device.installation_date.clone(), day, unit_id, globs);
            let error_msg = format!("Erro ao verificar medidor da laager no dia {:?}, {}, {}", day, laager_device.laager_code, err);
            eprintln!("{}", error_msg);
            write_to_log_file_thread(&error_msg, 0, "ERROR");
            return;
        }
    };

    match check_date(day) {
        Ok(true) => {
            // Rota de consumo de água por hora buscar no API-Server
            match ApiServer::get_laager_history_list(&laager_device.laager_code, day, globs).await {
                Ok(mut cons) => {
                    normalize_laager_consumption(&mut cons);

                    if let Some(history) = cons.iter().find(|hist| hist.date == day) {
                        let compiled_laager_data = compile_laager_data(history, day);
                        insert_data_laager_per_hour(
                            &laager_device.laager_code,
                            "Laager",
                            unit_id,
                            &compiled_laager_data,
                            laager_device.installation_date.as_deref(),
                            globs).await;
                    } else {
                        forecast_without_consumption_by_day(laager_device.installation_date.clone(), day, unit_id, globs);
                        eprintln!("Dispositivo Laager: {} sem histórico para o dia: {}", laager_device.laager_code, day);
                    }
                }
                Err(err) => { 
                    forecast_without_consumption_by_day(laager_device.installation_date.clone(), day, unit_id, globs);
                    eprintln!("Erro ao obter consumo: {}", err);
                    return;
                }
            };
        },
        Ok(false) => {
            match LaagerApi::get_laager_consumption(rf_device_id, globs).await {
                Ok(mut cons) => {
                    normalize_laager_consumption(&mut cons);
        
                    if let Some(history) = cons.iter().find(|hist| hist.date == day) {
                        let compiled_laager_data = compile_laager_data(history, day);
                        insert_data_laager_per_hour(
                            &laager_device.laager_code,
                            "Laager",
                            unit_id,
                            &compiled_laager_data,
                            laager_device.installation_date.as_deref(),
                             globs).await;
                    } else {
                        forecast_without_consumption_by_day(laager_device.installation_date.clone(), day, unit_id, globs);
                        eprintln!("Dispositivo Laager: {} sem histórico para o dia: {}", laager_device.laager_code, day);
                    }
                }
                Err(err) => { 
                    forecast_without_consumption_by_day(laager_device.installation_date.clone(), day, unit_id, globs);
                    eprintln!("Erro ao obter consumo: {}", err);
                    return;
                }
            };
        },
        Err(e) => println!("Erro ao processar a data: {}", e),
    }
    
}

fn check_date(day: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let input_date = NaiveDate::parse_from_str(day, "%Y-%m-%d")?;
    let today = Utc::now().date_naive();
    let duration = today.signed_duration_since(input_date).num_days().abs();
    Ok(duration > 60)
}
