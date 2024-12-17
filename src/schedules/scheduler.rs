
use std::collections::VecDeque;
use std::fs::OpenOptions;
use std::io::Write;
use std::str::FromStr;
use chrono::{Datelike, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use tokio::task;
use crate::db::entities::clients::{get_client, insert_data_client};
use crate::db::entities::units::{get_unit, insert_data_unit, update_unit};
use crate::external_api::api_server::ApiServer;
use crate::models::database_models::clients::Clients;
use crate::models::database_models::units::Units;
use crate::models::external_models::client::ClientInfo;
use crate::models::external_models::unit::UnitInfo;
use crate::GlobalVars;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use super::chiller::chiller_hx_parameters::process_chiller_hx_devices;
use super::chiller::chiller_xa_parameters::{process_chiller_xa_devices, process_chiller_xa_hvar_devices};
use super::energy::{process_energy_devices, reprocess_energy_forecast_view};
use super::waters::process_waters_devices;
use super::energy_efficiency::{process_energy_efficiency_dacs, process_energy_efficiency_duts};
use super::disponibility::{process_disponibility_dacs_devices, process_disponibility_dals_devices, process_disponibility_dams_devices, process_disponibility_dmts_devices, process_disponibility_dris_devices, process_disponibility_duts_devices};
use super::devices_l1_totalization::{process_l1_totalization_dacs, process_l1_totalization_duts};

pub async fn run_nightly_tasks(globs: &Arc<GlobalVars>, day: &str, units_with_others_timezones: Option<bool>, script_type: &str, client_ids: Option<Vec<i32>>, unit_ids: Option<Vec<i32>>) {
    let msg = "Começando Processamento";
    write_to_log_file_thread(msg, 0, "SCHEDULER");
    println!("{}", msg);

    let clients_result = ApiServer::get_clients(client_ids, globs).await;
    match clients_result {
        Ok(clients) => {
            let clients_queue: VecDeque<_> = clients.into_iter().collect();
            let clients_mutex = Arc::new(Mutex::new(clients_queue));
            
            let pool_size = 4;
            let mut handles = Vec::with_capacity(pool_size);
            for index in 0..pool_size {
                println!("Processando Dia: {}, thread {}", &day, index);
                write_to_log_file_thread(&format!("Processando Dia: {}, thread {}", &day, index), index, "SCHEDULER");
                let clients_mutex = clients_mutex.clone();
                let globs_clone = globs.clone();
                let unit_ids_clone = unit_ids.clone();
                let day_clone = day.to_owned();
                let script_type = script_type.to_owned();
                let handle = task::spawn(async move {
                    loop {
                        let client;
                        {
                            let mut clients = clients_mutex.lock().unwrap();
                            if clients.is_empty() {
                                println!("Encerrando thread {} do dia {}", index, day_clone);
                                write_to_log_file_thread(&format!("Encerrando thread {} do dia {}", index, day_clone), index, "SCHEDULER");
                                break;
                            }
                            client = clients.pop_front().unwrap();
                        }
                        
                        process_clients_range(client, &globs_clone, index, &day_clone, units_with_others_timezones, &script_type, unit_ids_clone.clone()).await;
                    }
                });
                handles.push(handle);
            }

            for handle in handles {
                if let Err(err) = handle.await {
                    let err_msg = format!("Thread error: {}", err);
                    write_to_log_file_thread(&err_msg, 0, "ERROR-THREAD");
                    eprintln!("{}", err_msg);
                }
            }
        }
        Err(err) => {
            write_to_log_file_thread(&err, 0, "ERROR");
            eprintln!("{}", err);
        }
    }

    reprocess_energy_forecast_view(day, globs);
}

pub async fn run_scheduler_many_days(start_date: &str, end_date: &str, client_ids: Option<Vec<i32>>, unit_ids: Option<Vec<i32>>, script_type: &str, globs: &Arc<GlobalVars>) {
    let start_date: NaiveDate = NaiveDate::from_str(start_date).unwrap_or_default();
    let end_date = NaiveDate::from_str(end_date).unwrap_or_default();

    let num_days = (end_date - start_date).num_days() + 1;

    let mut days = Vec::with_capacity(num_days as usize);

    for i in 0..num_days {
        let date = start_date + Duration::days(i);
        days.push(date.to_string());
    }

    let start = Instant::now();
    for day in days {
        run_nightly_tasks(globs, &day, None, script_type, client_ids.clone(), unit_ids.clone()).await;
    }

    let duration = start.elapsed();
    let msg = format!("Tempo decorrido script: {} segundos", duration.as_secs());
    write_to_log_file_thread(&msg, 0, "SCHEDULER");
    println!("{}", msg);
}

pub async fn start_scheduler(globs: &Arc<GlobalVars>, start_hour: u32) {
     loop {
        let next_start_time = Utc::now().date().and_hms_opt(start_hour, 15, 0).unwrap();
        let now = Utc::now();
        let units_with_others_timezones = if start_hour == 3 { false } else { true };    
        let mut duration_until_start = next_start_time - now;
    
        if duration_until_start.num_seconds() <= 0 {
            duration_until_start = next_start_time + Duration::days(1) - now;
        }
        
        let msg_init = format!("Rodará em {} segundos", duration_until_start.num_seconds());
        write_to_log_file_thread(&msg_init, 0, "SCHEDULER");
        println!("{}", msg_init);
        
        tokio::time::sleep(tokio::time::Duration::from_secs(duration_until_start.num_seconds() as u64)).await;
        
        let start = Instant::now();

        let day: String = (Utc::now() - chrono::Duration::days(1)).format("%Y-%m-%d").to_string();
        run_nightly_tasks(globs, &day, Some(units_with_others_timezones), "all", None, None).await;

        let duration = start.elapsed();

        let msg_time = format!("Tempo decorrido script: {} segundos", duration.as_secs());
        write_to_log_file_thread(&msg_time, 0, "SCHEDULER");
        println!("{}", msg_time);

    }
}

async fn process_clients_range(client: ClientInfo, globs: &Arc<GlobalVars>, thread: usize, day: &str, units_with_others_timezones: Option<bool>, script_type: &str, unit_ids: Option<Vec<i32>>) {
    process_client_units(&client, day, globs, thread, units_with_others_timezones, unit_ids, script_type).await;
}

async fn process_client_units(client_info: &ClientInfo, day: &str, globs: &Arc<GlobalVars>, thread: usize, units_with_others_timezones: Option<bool>, unit_ids: Option<Vec<i32>>, script_type: &str) {
    match verify_insert_client(client_info, globs) {
     Ok(client_db) => {
         let units_result = ApiServer::get_all_units_by_client(&client_info.client_id, units_with_others_timezones, unit_ids, day, globs).await;
         match units_result {
             Ok(units) => {
                 let start_message = format!("Começou cliente: {:?}-{:?}", client_info.client_id, &client_info.client_name);
                 write_to_log_file_thread(&start_message, thread, "INFO");

                 match script_type {
                    "energy" => {
                        for unit in units {
                            process_unit_energy_devices(client_db.0, unit, client_db.1, day, globs, thread, Some(false)).await;
                        }
                    },
                    "chiller" => {
                        for unit in units {
                            process_unit_chiller_devices(client_db.0, unit, client_db.1, day, globs, thread).await;
                        }
                    },
                    "water" => {
                        for unit in units {
                            process_unit_water_devices(client_db.0, client_db.1, unit, day, globs, thread).await;
                        }
                    },
                    "without_energy" => {
                        for unit in units {
                            process_unit_devices_script(client_db.0, unit, client_db.1, day, globs, thread).await;
                        } 
                    },
                    "energy_demand" => {
                        for unit in units {
                            process_unit_energy_devices(client_db.0, unit, client_db.1, day, globs, thread, Some(true)).await;
                        }
                    },
                    "energy_efficiency" => {
                        for unit in units {
                            process_unit_energy_efficiency_devices(client_db.0, client_db.1, unit, day, globs, thread).await;
                        } 
                    },
                    "process_unit_on_outside_programming" => {
                        for unit in units {
                            process_unit_on_outside_programming_devices(client_db.0, client_db.1, unit, day, globs, thread).await;
                        } 
                    },
                    _ => {
                        for unit in units {
                            process_unit_devices(client_db.0, client_db.1, unit, day, globs, thread).await;
                        } 
                    }
                };

                 
                 let end_message = format!("Encerrou cliente: {:?}-{:?}", client_info.client_id, &client_info.client_name);
                 write_to_log_file_thread(&end_message, thread, "INFO");
             }
             Err(err) => {
                 let error_msg = format!("Erro ao obter as unidades do cliente: {}, no dia: {}, {}", client_info.client_id, &day, err);
                 eprintln!("{}", error_msg);
                 write_to_log_file_thread(&error_msg, thread, "ERROR");
             }
         }
     },
     Err(err) => {
         let error_msg = format!("Erro ao processar Clientes no dia: {}, {}", &day, err);
         eprintln!("{}", error_msg);
         write_to_log_file_thread(&error_msg, thread, "ERROR");
     }
    } 
 }
 
fn verify_insert_client(client_info: &ClientInfo, globs: &Arc<GlobalVars>) -> Result<(i32, Option<i32>), String>{
    let has_client = get_client(&client_info.client_name, globs);

    let client = match has_client {
        Ok(res) => { res },
        Err(err) => return Err(format!("Erro ao verificar cliente, {}", err))
    };

    match client {
        Some(res) => {
            Ok((res.id.unwrap(), res.amount_minutes_check_offline))
        },
        None => {
            let client = Clients {
                id: None,
                client_name: client_info.client_name.clone(),
                amount_minutes_check_offline: None,
            };
    
            let inserted_info = insert_data_client(client, globs).unwrap();
            Ok(inserted_info)
        }
    }
}

async fn process_unit_devices(client_id: i32, client_minutes_to_check_offline: Option<i32>, unit_info: UnitInfo, day: &str, globs: &Arc<GlobalVars>, thread: usize) {
    match verify_insert_update_units(client_id, &unit_info, globs) {
        Ok(unit_id) => {
            let devices_result = ApiServer::get_config_devices(&unit_info.unit_id, day, globs).await;
            match devices_result {
                Ok(devices_config) => {
                    tokio::join!(
                        process_waters_devices(unit_id, devices_config.devices.laager_device, devices_config.devices.dma_device, day, client_minutes_to_check_offline, globs),
                        process_energy_devices(unit_id, &devices_config.devices.energy_devices, day, Some(false), client_minutes_to_check_offline, globs),
                        process_l1_totalization_dacs(unit_id, day, &devices_config.devices.dacs_to_l1_automation, client_minutes_to_check_offline, globs),
                        process_l1_totalization_duts(unit_id, day, &devices_config.devices.duts_to_l1_automation, client_minutes_to_check_offline, globs),
                        process_energy_efficiency_dacs(unit_id, day, &devices_config.devices.dacs_devices, client_minutes_to_check_offline, globs),
                        process_energy_efficiency_duts(unit_id, day, &devices_config.devices.duts_devices, client_minutes_to_check_offline, globs),
                        process_disponibility_duts_devices(unit_id, day, &devices_config.devices.duts_to_disponibility, client_minutes_to_check_offline, globs),
                        process_disponibility_dacs_devices(unit_id, day, &devices_config.devices.dacs_to_disponibility, client_minutes_to_check_offline, globs),
                        process_disponibility_dris_devices(unit_id, day, &devices_config.devices.dris_to_disponibility, client_minutes_to_check_offline, globs),
                        process_disponibility_dmts_devices(unit_id, day, &devices_config.devices.dmts_to_disponibility, client_minutes_to_check_offline, globs),
                        process_disponibility_dals_devices(unit_id, day, &devices_config.devices.dals_to_disponibility, client_minutes_to_check_offline, globs),
                        process_disponibility_dams_devices(unit_id, day, &devices_config.devices.dams_to_disponibility, client_minutes_to_check_offline, globs)
                    );
                }
                Err(err) => {
                    let error_msg = format!("Erro ao obter os dispositivos da unidade: {}, no dia {}, {}", unit_info.unit_id, &day, err);
                    eprintln!("{}", error_msg);
                    write_to_log_file_thread(&error_msg, thread, "ERROR");
                }
            }
        },
        Err(err) => {
            let error_msg = format!("Erro ao processar Unidades no dia {}, {}", &day, err);
            eprintln!("{}", error_msg);
            write_to_log_file_thread(&error_msg, thread, "ERROR");
        }
    }
}

async fn process_unit_devices_script(client_id: i32, unit_info: UnitInfo, client_minutes_to_check_offline: Option<i32>, day: &str, globs: &Arc<GlobalVars>, thread: usize) {
    match verify_insert_update_units(client_id, &unit_info, globs) {
        Ok(unit_id) => {
            let devices_result = ApiServer::get_config_devices(&unit_info.unit_id, day, globs).await;
            match devices_result {
                Ok(devices_config) => {
                    // sem medidor de energia
                    tokio::join!(
                        process_waters_devices(unit_id, devices_config.devices.laager_device, devices_config.devices.dma_device, day, client_minutes_to_check_offline, globs),
                        process_l1_totalization_dacs(unit_id, day, &devices_config.devices.dacs_to_l1_automation, client_minutes_to_check_offline, globs),
                        process_l1_totalization_duts(unit_id, day, &devices_config.devices.duts_to_l1_automation, client_minutes_to_check_offline, globs),
                        process_energy_efficiency_dacs(unit_id, day, &devices_config.devices.dacs_devices, client_minutes_to_check_offline, globs),
                        process_energy_efficiency_duts(unit_id, day, &devices_config.devices.duts_devices, client_minutes_to_check_offline, globs),
                        process_disponibility_duts_devices(unit_id, day, &devices_config.devices.duts_to_disponibility, client_minutes_to_check_offline, globs),
                        process_disponibility_dacs_devices(unit_id, day, &devices_config.devices.dacs_to_disponibility, client_minutes_to_check_offline, globs),
                        process_disponibility_dris_devices(unit_id, day, &devices_config.devices.dris_to_disponibility, client_minutes_to_check_offline, globs),
                        process_disponibility_dmts_devices(unit_id, day, &devices_config.devices.dmts_to_disponibility, client_minutes_to_check_offline, globs),
                        process_disponibility_dals_devices(unit_id, day, &devices_config.devices.dals_to_disponibility, client_minutes_to_check_offline, globs),
                        process_disponibility_dams_devices(unit_id, day, &devices_config.devices.dams_to_disponibility, client_minutes_to_check_offline, globs)
                    );
                }
                Err(err) => {
                    let error_msg = format!("Erro ao obter os dispositivos da unidade: {}, no dia {}, {}", unit_info.unit_id, &day, err);
                    eprintln!("{}", error_msg);
                    write_to_log_file_thread(&error_msg, thread, "ERROR");
                }
            }
        },
        Err(err) => {
            let error_msg = format!("Erro ao processar Unidades no dia {}, {}", &day, err);
            eprintln!("{}", error_msg);
            write_to_log_file_thread(&error_msg, thread, "ERROR");
        }
    }
}

async fn process_unit_energy_devices(client_id: i32, unit_info: UnitInfo, client_minutes_to_check_offline: Option<i32>, day: &str, globs: &Arc<GlobalVars>, thread: usize, only_demand: Option<bool>) {
    match verify_insert_update_units(client_id, &unit_info, globs) {
        Ok(unit_id) => {
            let devices_result = ApiServer::get_config_devices(&unit_info.unit_id, day, globs).await;
            match devices_result {
                Ok(devices_config) => {
                    process_energy_devices(unit_id, &devices_config.devices.energy_devices, day, only_demand, client_minutes_to_check_offline, globs).await;
                }
                Err(err) => {
                    let error_msg = format!("SCRIPT Energia - Erro ao obter os dispositivos da unidade: {}, no dia {}, {}", unit_info.unit_id, &day, err);
                    eprintln!("{}", error_msg);
                    write_to_log_file_thread(&error_msg, thread, "ERROR");
                }
            }
        },
        Err(err) => {
            let error_msg = format!("SCRIPT Energia - Erro ao processar Unidades no dia {}, {}", &day, err);
            eprintln!("{}", error_msg);
            write_to_log_file_thread(&error_msg, thread, "ERROR");
        }
    }
}

async fn process_unit_chiller_devices(client_id: i32, unit_info: UnitInfo, client_minutes_to_check_offline: Option<i32>, day: &str, globs: &Arc<GlobalVars>, thread: usize) {
    match verify_insert_update_units(client_id, &unit_info, globs) {
        Ok(unit_id) => {
            let devices_result = ApiServer::get_config_devices(&unit_info.unit_id, day, globs).await;
            match devices_result {
                Ok(devices_config) => {
                    tokio::join!(
                        process_chiller_hx_devices(unit_id, &devices_config.devices.dris_to_disponibility, day, client_minutes_to_check_offline, globs),
                        process_chiller_xa_devices(unit_id, &devices_config.devices.dris_to_disponibility, day, client_minutes_to_check_offline, globs),
                        process_chiller_xa_hvar_devices(unit_id, &devices_config.devices.dris_to_disponibility, day, client_minutes_to_check_offline, globs),
                    );
                }
                Err(err) => {
                    let error_msg = format!("SCRIPT CHILLER - Erro ao obter os dispositivos da unidade: {}, no dia {}, {}", unit_info.unit_id, &day, err);
                    eprintln!("{}", error_msg);
                    write_to_log_file_thread(&error_msg, thread, "ERROR");
                }
            }
        },
        Err(err) => {
            let error_msg = format!("SCRIPT CHILLER - Erro ao processar Unidades no dia {}, {}", &day, err);
            eprintln!("{}", error_msg);
            write_to_log_file_thread(&error_msg, thread, "ERROR");
        }
    }
}

async fn process_unit_water_devices(client_id: i32, client_minutes_to_check_offline: Option<i32>, unit_info: UnitInfo, day: &str, globs: &Arc<GlobalVars>, thread: usize) {
    match verify_insert_update_units(client_id, &unit_info, globs) {
        Ok(unit_id) => {
            let devices_result = ApiServer::get_config_devices(&unit_info.unit_id, day, globs).await;
            match devices_result {
                Ok(devices_config) => {
                    process_waters_devices(unit_id, devices_config.devices.laager_device, devices_config.devices.dma_device, day, client_minutes_to_check_offline, globs).await;
                }
                Err(err) => {
                    let error_msg = format!("SCRIPT Água - Erro ao obter os dispositivos da unidade: {}, no dia {}, {}", unit_info.unit_id, &day, err);
                    eprintln!("{}", error_msg);
                    write_to_log_file_thread(&error_msg, thread, "ERROR");
                }
            }
        },
        Err(err) => {
            let error_msg = format!("SCRIPT Água - Erro ao processar Unidades no dia {}, {}", &day, err);
            eprintln!("{}", error_msg);
            write_to_log_file_thread(&error_msg, thread, "ERROR");
        }
    }
    
}

async fn process_unit_energy_efficiency_devices(client_id: i32, client_minutes_to_check_offline: Option<i32>, unit_info: UnitInfo, day: &str, globs: &Arc<GlobalVars>, thread: usize) {
    match verify_insert_update_units(client_id, &unit_info, globs) {
        Ok(unit_id) => {
            let devices_result = ApiServer::get_config_devices(&unit_info.unit_id, day, globs).await;
            match devices_result {
                Ok(devices_config) => {
                    tokio::join!(
                        process_energy_efficiency_dacs(unit_id, day, &devices_config.devices.dacs_devices, client_minutes_to_check_offline, globs),
                        process_energy_efficiency_duts(unit_id, day, &devices_config.devices.duts_devices, client_minutes_to_check_offline, globs),
                    );                }
                Err(err) => {
                    let error_msg = format!("SCRIPT Eficiência Energética - Erro ao obter os dispositivos da unidade: {}, no dia {}, {}", unit_info.unit_id, &day, err);
                    eprintln!("{}", error_msg);
                    write_to_log_file_thread(&error_msg, thread, "ERROR");
                }
            }
        },
        Err(err) => {
            let error_msg = format!("SCRIPT Eficiência Energética - Erro ao processar Unidades no dia {}, {}", &day, err);
            eprintln!("{}", error_msg);
            write_to_log_file_thread(&error_msg, thread, "ERROR");
        }
    }
    
}

async fn process_unit_on_outside_programming_devices(client_id: i32, client_minutes_to_check_offline: Option<i32>, unit_info: UnitInfo, day: &str, globs: &Arc<GlobalVars>, thread: usize) {
    match verify_insert_update_units(client_id, &unit_info, globs) {
        Ok(unit_id) => {
            let devices_result = ApiServer::get_config_devices(&unit_info.unit_id, day, globs).await;
            match devices_result {
                Ok(devices_config) => {
                    tokio::join!(
                        process_l1_totalization_dacs(unit_id, day, &devices_config.devices.dacs_to_l1_automation, client_minutes_to_check_offline, globs),
                        process_l1_totalization_duts(unit_id, day, &devices_config.devices.duts_to_l1_automation, client_minutes_to_check_offline, globs),
                    );                }
                Err(err) => {
                    let error_msg = format!("SCRIPT Tempo Fora da Programação - Erro ao obter os dispositivos da unidade: {}, no dia {}, {}", unit_info.unit_id, &day, err);
                    eprintln!("{}", error_msg);
                    write_to_log_file_thread(&error_msg, thread, "ERROR");
                }
            }
        },
        Err(err) => {
            let error_msg = format!("SCRIPT Tempo Fora da Programação - Erro ao processar Unidades no dia {}, {}", &day, err);
            eprintln!("{}", error_msg);
            write_to_log_file_thread(&error_msg, thread, "ERROR");
        }
    }
    
}

fn verify_insert_update_units(client_id: i32, unit_info: &UnitInfo, globs: &Arc<GlobalVars>) -> Result<i32, String>{
    let actual_unit_info = get_unit(unit_info.unit_id, globs);

    let unit = match actual_unit_info {
        Ok(unitData) => unitData,
        Err(err) => return Err(format!("Erro ao verificar cliente, {}", err))
    };

    if let Some(unitData) = unit {
        if unitData.unit_name != unit_info.unit_name || 
            unitData.city_name != unit_info.city_name || 
            unitData.state_name != unit_info.state_name || 
            unitData.tarifa_kwh != unit_info.tarifa_kwh ||
            unitData.constructed_area != unit_info.constructed_area ||
            unitData.capacity_power != unit_info.capacity_power {
            update_unit(&unit_info, globs);
        }
        return Ok(unitData.id.unwrap());
    } else {
        let unit = Units {
            id: None,
            client_id,
            unit_name: unit_info.unit_name.clone(),
            reference_id: unit_info.unit_id,
            city_name: unit_info.city_name.clone(),
            state_name: unit_info.state_name.clone(),
            tarifa_kwh: unit_info.tarifa_kwh.clone(),
            constructed_area: unit_info.constructed_area.clone(),
            capacity_power: unit_info.capacity_power.clone()
        };

       let inserted_id = insert_data_unit(unit, globs).unwrap();
       Ok(inserted_id)
    }
}

//auxiliar testes
pub fn write_to_log_file_thread(message: &str, thread: usize, tag: &str) {
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(format!("log-thread-{}.txt", thread)) {
        let json_str = serde_json::json!({ "msg": message, "tag": tag, "tslog": Local::now().format("%d/%m/%y %H:%M:%S").to_string() }).to_string();
        if let Err(e) = writeln!(file, "{}", json_str) {
            eprintln!("Erro ao escrever no arquivo de log: {}", e);
        }
    } else {
        eprintln!("Erro ao abrir o arquivo de log.");
    }
}
