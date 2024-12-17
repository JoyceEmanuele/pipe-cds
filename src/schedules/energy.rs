use std::collections::HashSet;
use std::error::Error;
use std::{collections::HashMap, sync::Arc};

use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::Serialize;
use serde_json;

use crate::db::entities::energy_consumption_forecast::{get_energy_consumption_target, insert_data_energy_consumption_forecast, process_energy_forecast_view, GetEnergyTarget};
use crate::compression::common_func::check_amount_minutes_offline;
use crate::schedules::device_disponibility::insert_device_disponibility_hist;
use crate::db::entities::energy_demand_minutes_hist::insert_data_demand;
use crate::db::entities::energy_hist::{get_energy_consumption_average, get_last_valid_consumption, get_total_days_unit_with_consumption, GetEnergyAverage};
use crate::db::entities::energy_monthly_consumption_target::{insert_data_energy_monthly_consumption_target, monthly_target_exists_for_unit};
use crate::models::database_models::energy_monthly_consumption_target;
use crate::schedules::scheduler::write_to_log_file_thread;
use crate::telemetry_payloads::energy::padronized::EnergyDemandTelemetry;
use crate::http::structs::energy::{GetDayEnergyConsumptionResponse, GetHourEnergyConsumptionResponse, GetLastValidConsumption, ParamsGetTotalDaysConsumptionUnit};
use crate::{app_history::{compiler_queues::{task_queue_manager, CompilationRequest}, energy_hist::{CompiledEnergyData, EnergyDataStruct, EnergyHist, EnergyHistParams, HoursCompiledEnergyData}}, db::entities::{electric_circuits::{get_electric_circuit, insert_data_electric_circuits, update_electric_circuit}, energy_hist::insert_data_energy}, http::structs::energy::GetEnergyConsumptionResponse, models::{database_models::electric_circuits::ElectricCircuit, external_models::device::EnergyDevice}, telemetry_payloads::energy::padronized::PadronizedEnergyTelemetry, GlobalVars};
use crate::models::database_models::{energy_consumption_forecast, energy_demand_minutes_hist, energy_hist};

#[derive(Debug, Serialize)]
pub struct EnergyConsumptionPerDay {
    pub day: String,
    pub total_measured: f64,
    pub max_day_total_measured: f64,
    pub hours: Option<Vec<HourlyConsumption>>,
    pub electric_circuit_reference_id: i32,
    pub contains_invalid: bool,
    pub contains_processed: bool
}

#[derive(Debug, Serialize)]
pub struct HourlyConsumption {
    pub hour: String,
    pub totalMeasured: f64,
    pub dataIsInvalid: bool,
    pub dataIsProcessed: bool
}

pub async fn process_energy_devices(unit_id: i32, energy_devices: &Option<Vec<EnergyDevice>>, day: &str, only_demand: Option<bool>, client_minutes_to_check_offline: Option<i32>, globs: &Arc<GlobalVars>) {
    if let Some(devices) = energy_devices {

        for energy_device in devices {
            let params = EnergyHistParams::parse_parameters(energy_device, day);
            let mut params_clone = params.clone();
            
            let response = match task_queue_manager(CompilationRequest::EnergyQuery(params), globs).await {
                Ok(response) => response,
                Err(err) => {
                    insert_device_disponibility_hist(unit_id, Decimal::new(0,0), day, &params_clone.energy_device_id, globs);
                    continue;
                }
            };

            let response_data = match serde_json::from_str::<EnergyHist>(&response) {
                Ok(resp) => resp,
                Err(err) => {
                    insert_device_disponibility_hist(unit_id, Decimal::new(0,0), day, &params_clone.energy_device_id, globs);
                    continue;
                },
            };

            verify_hours_online(response_data.clone(), client_minutes_to_check_offline, energy_device.dri_interval, params_clone.start_time, &params_clone.energy_device_id, unit_id, day, globs);

            match verify_insert_update_electric_circuits(unit_id, energy_device, globs) {
                Ok(electric_circuit_id) => {
                    if !only_demand.unwrap_or(false) {
                        let compiled_energy_data = compile_energy_data(&response_data.clone(), day);
                        insert_data_energy_per_hour(electric_circuit_id, &compiled_energy_data, &mut params_clone, unit_id, energy_device, globs).await;
                    }

                    let grouped_telemetries_demand = group_telemetries_by_15_minutes(response_data.data.clone(), day);
                    let grouped_averages = calculate_group_telemetries(&grouped_telemetries_demand);
                    insert_demand_hist(grouped_averages, electric_circuit_id, globs);
                }
                Err(err) => {
                    continue;
                }
            };
        }
        calc_consumption_monthly_target(unit_id, day, devices.to_vec(), globs);
    }
}

fn compile_energy_data(energy_hist: &EnergyHist, day: &str) -> CompiledEnergyData {
    let start_time = NaiveDateTime::parse_from_str(&format!("{}T00:00:00", day), "%Y-%m-%dT%H:%M:%S").unwrap();
    let end_time = start_time + Duration::days(1);
    let sample_with_params = energy_hist.data.iter().find(|&x| {
        if let Some(timestamp) = x.timestamp {
            timestamp >= start_time && timestamp < end_time && EnergyHist::num_fields_with_value(x) > 1
        } else {
            false
        }
    });

    let mut data_struct = generate_data_struct(day, sample_with_params);

    format_data(&mut data_struct, energy_hist)
}

fn generate_data_struct(day: &str, sample: Option<&PadronizedEnergyTelemetry>) -> EnergyDataStruct {
    let mut result = EnergyDataStruct::new(day);

    if sample.is_some() {
        for hour in 0..24 {
            let hour_str = format!("{:02}", hour);
            result.hour_values.insert(hour_str.clone(), Vec::new());
            result.hours.push(hour_str.clone());
        }
    }

    result
}

fn format_data(data_struct: &mut EnergyDataStruct, energy_hist: &EnergyHist) -> CompiledEnergyData {
    let data = &energy_hist.data;

    for hist in data {
        if hist.en_at_tri.is_some() {
            let (hour) = {
                let timestamp = hist.timestamp.unwrap();
                let hour = timestamp.time().hour();
                let hour_str = format!("{:02}", hour); 

                hour_str
            };

            match data_struct.hour_values.get_mut(&hour[..2]) {
                Some(vec) => {
                    if let Some(value) = hist.en_at_tri {
                        vec.push(value);
                    } else {
                        let msg = format!("en_at_tri is absent. {:?}", energy_hist.energy_device_id);
                        write_to_log_file_thread(&msg, 0, "WARN");

                    }
                }
                None => {
                    let msg = format!("The key {:?} was not found in `hour_values`. {:?}, {:?}", &hour[..2], energy_hist.energy_device_id, data_struct);
                    write_to_log_file_thread(&msg, 0, "WARN");
                }
            }
        } else {
            continue;
        }
    }

    let mut obj = CompiledEnergyData::new(&data_struct);
    let mut hours_sorted: Vec<&String> = data_struct.hour_values.keys().collect();
    hours_sorted.sort_by(|a, b| a.cmp(b));
    let mut zeroPositions: Vec<usize> = Vec::new();
    let mut currentPosition = 0;
    let mut last_non_zero_index: Option<usize> = None;

    for (hour_index, hour) in hours_sorted.iter().enumerate() {
        let hour_data_vec = data_struct.hour_values.get(&hour.to_string()).unwrap();
        let next_hour_data_vec: Option<Vec<f64>> = 
        if *hour != "23" {
            data_struct.hour_values.get(&format!("{:02}", hour_index + 1)).cloned()
        } else {
            None
        };

        let hour_data_vec = if hour_data_vec.contains(&-1.0) {
            vec![]
        } else {
            hour_data_vec.clone()
        };
        
        let next_hour_data_vec = match next_hour_data_vec {
            Some(vec) => if vec.contains(&-1.0) {
                Some(vec![])
            } else {
                Some(vec)
            },
            None => None,
        };

        let hour_data_value = if hour_data_vec.len() > 0 {
            hour_data_vec[hour_data_vec.len() - 1] - hour_data_vec.first().copied().unwrap_or(0.0)
        } else {
            hour_data_vec.first().copied().unwrap_or(0.0)
        };

        let mut last_en_at_tri = hour_data_vec.last().copied().unwrap_or(0.0);

        let next_hour_value = if let Some(next_data) = &next_hour_data_vec {
            if next_data.len() > 0 {
                last_en_at_tri = next_data[0];
                next_data[0] - hour_data_vec.first().copied().unwrap_or(0.0)
            } else {
                hour_data_value
            }
        } else {
            hour_data_value
        }; 

        let total_measured = if !hour_data_vec.is_empty() { next_hour_value } else { 0.0 };
        obj.hours.push(HoursCompiledEnergyData{
            hour: hour.to_string(),
            total_measured,
            last_en_at_tri_hour: last_en_at_tri,
            first_en_at_tri_hour: hour_data_vec.first().copied().unwrap_or(0.0),
            is_measured_consumption: false,
            is_valid_consumption: verify_is_valid_consumption(total_measured, hour_data_vec, next_hour_data_vec),
        });

        let mut total_measured = 0.0;

        for hour_data in &obj.hours {
            total_measured += hour_data.total_measured;
        }
        
        obj.total_measured = total_measured;
    }

    let mut formatedEnergyData = obj.clone();

    while currentPosition < obj.hours.len() {
        if !obj.hours[currentPosition].is_valid_consumption {
            
            let mut position = currentPosition;
            while position < obj.hours.len() && !obj.hours[position].is_valid_consumption {
                if last_non_zero_index.is_some() && last_non_zero_index.unwrap() < currentPosition {
                    let _ = zeroPositions.push(position);
                }
                position += 1;
            }
            currentPosition = position;
        } else {
            if let Some(last_index) = last_non_zero_index {
                if !zeroPositions.is_empty() {
                    let last_en_at_tri = obj.hours[last_index].last_en_at_tri_hour;
                    let next_first_en_at_tri = obj.hours[currentPosition].first_en_at_tri_hour;

                    let total_consumption = next_first_en_at_tri - last_en_at_tri;
                    let calculated_total_measured = total_consumption / (zeroPositions.len() as f64);

                    for &i in &zeroPositions {
                        formatedEnergyData.hours[i].total_measured = calculated_total_measured;
                        formatedEnergyData.hours[i].is_measured_consumption = true;
                        formatedEnergyData.hours[i].is_valid_consumption = calculated_total_measured >= 0.0 && calculated_total_measured < 1000.0;
                    }
    
                    zeroPositions.clear();
                }
            }
            last_non_zero_index = Some(currentPosition);
            currentPosition += 1;
        }
    }

    formatedEnergyData
}

async fn insert_data_energy_per_hour(electric_circuit_id: i32, energy_data: &CompiledEnergyData, parameters: &mut EnergyHistParams, unit_id: i32, energy_device: &EnergyDevice, globs: &Arc<GlobalVars>) {
    let mut first_non_zero_history: Option<energy_hist::EnergyHist> = None;

    for hour in &energy_data.hours {
        let total_cons = format!("{:.2}", &hour.total_measured);

        let history = energy_hist::EnergyHist {
            electric_circuit_id,
            consumption: if hour.is_valid_consumption { Decimal::from_str_exact(&total_cons).unwrap() } else { Decimal::from(0) },
            record_date: NaiveDateTime::parse_from_str(&format!("{} {}:00:00", energy_data.day, hour.hour), "%Y-%m-%d %H:%M:%S").unwrap_or_default(),
            is_measured_consumption: hour.is_measured_consumption,
            is_valid_consumption: hour.is_valid_consumption,
        };

        if history.is_valid_consumption && !history.is_measured_consumption && first_non_zero_history.is_none() {
            first_non_zero_history = Some(history.clone());
        }
        
        insert_data_energy(history.clone(), globs);

        calc_energy_consumption_forecast(history.electric_circuit_id.clone(), history.record_date.clone(), globs)
    }

    if first_non_zero_history.is_some() {
        match verify_update_energy_consumption(&first_non_zero_history.unwrap(), parameters, electric_circuit_id, globs).await {
            Ok(res) => { },
            Err(err) => {eprintln!("Não foi possível verificar o último consumo válido: {:?}", err)}
        };
    }
}

fn verify_is_valid_consumption(consumption: f64, hour_data_vec: Vec<f64>, next_hour_data_vec: Option<Vec<f64>>) -> bool {
    if consumption < 0.0 || consumption > 1000.0 {
        false
    } else if hour_data_vec.is_empty() || (hour_data_vec.len() == 1 && next_hour_data_vec.unwrap_or([].to_vec()).is_empty()) {
        false
    } else {
        true
    }
}

fn verify_insert_update_electric_circuits(unit_id: i32, energy_device: &EnergyDevice, globs: &Arc<GlobalVars>) -> Result<i32, String> {
    let actual_eletric_circuit_info = get_electric_circuit(energy_device.electric_circuit_id, globs);

    let electric_circuit = match actual_eletric_circuit_info {
        Ok(electric_circuit_tuple) => electric_circuit_tuple,
        Err(err) => return Err(format!("Erro ao verificar Circuito elétrico, {}", err))
    };

    if let Some(electric_circuit_tuple) = electric_circuit {
        if electric_circuit_tuple.1 != energy_device.electric_circuit_name {
            update_electric_circuit(electric_circuit_tuple.0, &energy_device.electric_circuit_name, globs);
        }
        Ok(electric_circuit_tuple.0)
    } else {
        let electric_circuit = ElectricCircuit {
            id: None,
            unit_id,
            name: energy_device.electric_circuit_name.clone(),
            reference_id: energy_device.electric_circuit_id,
        };

       let inserted_id = insert_data_electric_circuits(electric_circuit, globs).unwrap();
       Ok(inserted_id)
    }
}

pub fn fill_consumption_by_day(consumption_day_list: Vec<GetDayEnergyConsumptionResponse>, start_date: NaiveDate, end_date: NaiveDate, isDielUser: bool) -> Vec<EnergyConsumptionPerDay> {
    let mut existing_dates_by_circuit: HashMap<i32, HashSet<NaiveDate>> = HashMap::new();

    for entry in consumption_day_list.iter() {
        existing_dates_by_circuit
            .entry(entry.electric_circuit_reference_id)
            .or_insert_with(HashSet::new)
            .insert(NaiveDate::parse_from_str(&entry.day, "%Y-%m-%d").unwrap());
    }

    let mut energy_consumption_list = Vec::new();

    for i in 0..=(end_date - start_date).num_days() {
        let current_date = start_date + Duration::days(i as i64);

        for (circuit_id, _existing_dates) in &existing_dates_by_circuit {
            let mut total_measured: f64 = 0.0;
            let mut max_day_total_measured = 0.0;
            let hours = None;
            let mut contains_invalid: bool = false;
            let mut contains_processed: bool = false;

            if let Some(existing_dates_set) = existing_dates_by_circuit.get(&circuit_id) {
                if existing_dates_set.contains(&current_date) {
                    if let Some(entry) = consumption_day_list.iter().find(|e| e.electric_circuit_reference_id == *circuit_id && e.day == current_date.format("%Y-%m-%d").to_string()) {
                        total_measured = entry.total_measured.to_f64().unwrap_or(0.0);
                        max_day_total_measured = entry.max_day_total_measured.to_f64().unwrap_or(0.0);

                        match isDielUser {
                            false => {
                                if ((entry.invalid_count as f64) > ((entry.readings_count as f64) * 0.10)) || ((entry.processed_count as f64) > ((entry.readings_count as f64) * 0.10)) {
                                    if entry.processed_count > entry.invalid_count {
                                        contains_processed = true;
                                    } else {
                                        contains_invalid = true;
                                    }                
                                }
                            },
                            true => {
                                if (entry.invalid_count > 0) || (entry.processed_count > 0) {
                                    if entry.processed_count > entry.invalid_count {
                                        contains_processed = true;
                                    } else {
                                        contains_invalid = true;
                                    }
                                }
                            }
                        };
                    }
                }
            }

            energy_consumption_list.push(EnergyConsumptionPerDay {
                day: current_date.format("%Y-%m-%d").to_string(),
                total_measured,
                max_day_total_measured,
                hours,
                electric_circuit_reference_id: *circuit_id,
                contains_invalid,
                contains_processed
            });
        }
    }

    energy_consumption_list.sort_by(|a, b| {
        let date_a = NaiveDate::parse_from_str(&a.day, "%Y-%m-%d").unwrap();
        let date_b = NaiveDate::parse_from_str(&b.day, "%Y-%m-%d").unwrap();
        date_a.cmp(&date_b)
    });

    energy_consumption_list
}

pub fn adjust_consumption_by_hour(day_consumption_list: Vec<EnergyConsumptionPerDay>, hour_consumption_list: Vec<GetHourEnergyConsumptionResponse>) -> Vec<EnergyConsumptionPerDay> {
    let mut hour_data_map: HashMap<String, Vec<GetHourEnergyConsumptionResponse>> = HashMap::new();
    
    for hour_data in hour_consumption_list {
        let hour_key = hour_data.hour.format("%Y-%m-%d %H").to_string();
        hour_data_map.entry(hour_key).or_insert_with(Vec::new).push(hour_data);
    }

    let mut adjusted_consumption: Vec<EnergyConsumptionPerDay> = Vec::new();

    for day_consumption in day_consumption_list.iter() {
        let mut hourly_consumption: Vec<HourlyConsumption> = Vec::new();
        
        for hour in 0..24 {
            let hour_key = format!("{} {:02}", day_consumption.day, hour);
            
            if let Some(hour_data) = hour_data_map.get(&hour_key) {
                let total_measured_hour: f64 = hour_data.iter()
                    .filter(|data| data.electric_circuit_reference_id == day_consumption.electric_circuit_reference_id)
                    .filter_map(|data| data.total_measured.to_f64())
                    .sum();
                hourly_consumption.push(HourlyConsumption {
                    hour: format!("{:02}", hour),
                    totalMeasured: total_measured_hour,
                    dataIsInvalid: hour_data[0].contains_invalid,
                    dataIsProcessed: hour_data[0].contains_processed
                });
            } else {
                hourly_consumption.push(HourlyConsumption {
                    hour: format!("{:02}", hour),
                    totalMeasured: 0.0,
                    dataIsInvalid: false,
                    dataIsProcessed: false
                });
            }
        }

        adjusted_consumption.push(EnergyConsumptionPerDay {
            day: day_consumption.day.clone(),
            total_measured: day_consumption.total_measured.to_f64().unwrap_or(0.0),
            max_day_total_measured: day_consumption.max_day_total_measured.to_f64().unwrap_or(0.0),
            hours: Some(hourly_consumption),
            electric_circuit_reference_id: day_consumption.electric_circuit_reference_id,
            contains_invalid: day_consumption.contains_invalid,
            contains_processed: day_consumption.contains_processed
        });
    }

    adjusted_consumption
}

async fn verify_update_energy_consumption(actual_history: &energy_hist::EnergyHist, parameters: &mut EnergyHistParams, electric_circuit_id: i32, globs: &Arc<GlobalVars>) -> Result <(), Box<dyn Error>>{
    let history = match get_last_valid_consumption(actual_history.electric_circuit_id, actual_history.record_date, globs) {
        Ok(hist) => hist,
        Err(err) => {
            eprintln!("Erro ao encontrar último registro de consumo de energia, {:?}", err);
            None
        }
    };

    if history.as_ref().is_some_and(|h| h.record_date != actual_history.record_date - Duration::hours(1)) {
        let history_clone = history.unwrap().clone();

        let saved_history: bool = match verify_telemetries_saved_data(parameters, (history_clone.clone().record_date + Duration::hours(1)).date(), (actual_history.clone().record_date - Duration::hours(1)).date(), electric_circuit_id, globs).await {
            Ok(_res) => true,
            Err(_err) => false,
        };

        let history_after_save_data = match get_last_valid_consumption(actual_history.electric_circuit_id, actual_history.record_date, globs) {
            Ok(hist) => hist,
            Err(err) => {
                eprintln!("Erro ao encontrar último registro de consumo de energia, {:?}", err);
                None
            }
        };

        if !saved_history || history_after_save_data.as_ref().is_some_and(|h| h.record_date != actual_history.record_date - Duration::hours(1)) {
            verify_update_energy_consumption_aux(parameters, actual_history, history_after_save_data.unwrap(), electric_circuit_id, globs).await?;
        }
    }

    Ok(())
}

async fn verify_update_energy_consumption_aux(parameters: &mut EnergyHistParams, actual_history: &energy_hist::EnergyHist, history: GetLastValidConsumption, electric_circuit_id: i32, globs: &Arc<GlobalVars>) -> Result <(), Box<dyn Error>> {
    let mut parameters_clone = parameters.clone();
    let history_clone = history.clone();
    
    parameters.start_time = history.record_date;
    parameters.end_time = parameters.start_time.with_minute(59).unwrap().with_second(59).unwrap();

    parameters_clone.start_time = actual_history.record_date;
    parameters_clone.end_time = parameters_clone.start_time.with_minute(59).unwrap().with_second(59).unwrap();


    let last_valid_consumption_telemetries = verify_telemetries(parameters, globs).await?;
    let last_valid_consumption_telemetries_data_filtered: Vec<PadronizedEnergyTelemetry> = last_valid_consumption_telemetries.data.into_iter().filter(|t| t.en_at_tri.unwrap_or(-1.0) >= 0.0).collect();

    let actual_telemetries = verify_telemetries(&mut parameters_clone, globs).await?;
    let actual_telemetries_data_filtered: Vec<PadronizedEnergyTelemetry> = actual_telemetries.data.into_iter().filter(|t| t.en_at_tri.unwrap_or(-1.0) >= 0.0).collect();
    
    // inicializando como -1, para casos onde após os novos filtros, não encontre telemetrias válidas
    let mut difference_consumption = -1.0;

    // casos em que as telemetrias foram consideradas válidas antes das novas validações,
    // ou seja, após os filtros aplicados nessas telemetrias, 
    //se algum valor que agora é desconsiderado tiver sido considerado como válido, não faremos a média do consumo
    if !actual_telemetries_data_filtered.is_empty() && !last_valid_consumption_telemetries_data_filtered.is_empty() {
        let actual_consumption = actual_telemetries_data_filtered[0].en_at_tri.unwrap_or(0.0);
        let last_valid_consumption = last_valid_consumption_telemetries_data_filtered[last_valid_consumption_telemetries_data_filtered.len() - 1].en_at_tri.unwrap_or(0.0);
        difference_consumption = actual_consumption - last_valid_consumption;

    }
    
    let hours_without_consumption = (actual_history.record_date - history_clone.record_date).num_hours() - 1;
    let formatted_consumption = difference_consumption.to_f64().unwrap_or(0.0) / hours_without_consumption as f64;

    for i in 1..hours_without_consumption + 1 {
        let date_time = history_clone.record_date + Duration::hours(i.into());
        let is_valid_consumption = formatted_consumption >= 0.0 && formatted_consumption < 1000.0;
        let history = energy_hist::EnergyHist {
            electric_circuit_id,
            consumption: if is_valid_consumption { Decimal::from_f64_retain(formatted_consumption).unwrap_or(Decimal::from(0)).round_dp(3) } else { Decimal::from(0) },
            record_date: date_time,
            is_measured_consumption: true,
            is_valid_consumption,
        };

        insert_data_energy(history.clone(), globs);

        calc_energy_consumption_forecast(history.electric_circuit_id.clone(), history.record_date.clone(), globs)
    }

    Ok(())
}

async fn verify_telemetries(parameters: &mut EnergyHistParams, globs: &Arc<GlobalVars>) -> Result<EnergyHist, Box<dyn Error>> {
    let response: String = match task_queue_manager(CompilationRequest::EnergyQuery(parameters.to_owned()), globs).await {
        Ok(response) => response,
        Err(err) => {
            return Err(format!("Erro ao executar requisição, {}", err).into());
        }
    };

    let response_data = match serde_json::from_str::<EnergyHist>(&response) {
        Ok(resp) => resp,
        Err(err) => {
            return Err(format!("Erro ao desserealizar JSON: {}", err).into());
        },
    };

    Ok(response_data)
}

fn group_telemetries_by_15_minutes(telemetries: Vec<PadronizedEnergyTelemetry>, day: &str) -> HashMap<NaiveDateTime, Vec<EnergyDemandTelemetry>> {
    let mut grouped_telemetries: HashMap<NaiveDateTime, Vec<EnergyDemandTelemetry>> = HashMap::new();
    for telemetry in telemetries.iter() {
        let Some(timestamp) = telemetry.timestamp else {
            continue
        };
        let Some(demand_med_at) = telemetry.demanda_med_at else {
            continue;
        };
        
        let timestamp_day = timestamp.date().format("%Y-%m-%d").to_string();
        if timestamp_day == day && timestamp.hour() == 0 && timestamp.minute() < 15 {
            continue;
        }

        let adjusted_timestamp = timestamp.checked_sub_signed(Duration::minutes(15)).unwrap_or(timestamp);
        let rounded_minute = ((adjusted_timestamp.minute() / 15) * 15) as u32;

        let final_timestamp = adjusted_timestamp.date().and_hms(adjusted_timestamp.hour(), rounded_minute, 0);

        let energy_demand_telemetry = EnergyDemandTelemetry {
            timestamp: final_timestamp,
            demanda_med_at: demand_med_at,
            min_demand: demand_med_at,
            max_demand: demand_med_at,
        };

        grouped_telemetries.entry(final_timestamp).or_insert_with(Vec::new).push(energy_demand_telemetry);
    }
    grouped_telemetries
}

fn calculate_group_telemetries(grouped_telemetries: &HashMap<NaiveDateTime, Vec<EnergyDemandTelemetry>>) -> Vec<EnergyDemandTelemetry> {
    let mut group_demands: Vec<EnergyDemandTelemetry> = Vec::new();

    for (time_interval, telemetry_array) in grouped_telemetries {
        let mut total_values: HashMap<&str, (f64, usize)> = HashMap::new();

        let mut min_demand = f64::MAX;
        let mut max_demand = f64::MIN;

        for telemetry in telemetry_array {
            let (sum, count_val) = total_values.entry("demanda_med_at").or_insert((0.0, 0));
            *sum += telemetry.demanda_med_at;
            *count_val += 1;

            if telemetry.demanda_med_at < min_demand {
                min_demand = telemetry.demanda_med_at;
            }
            if telemetry.demanda_med_at > max_demand {
                max_demand = telemetry.demanda_med_at;
            }
        }

        let mut grouped_telemetry = EnergyDemandTelemetry::new(*time_interval);

        if let Some((total, field_count)) = total_values.get("demanda_med_at") {
            let avg = total / *field_count as f64;
            let avg_rounded = (avg * 100.0).round() / 100.0;
            grouped_telemetry.set_average_demand(avg_rounded);
        }

        if min_demand != f64::MAX {
            grouped_telemetry.set_min_demand(min_demand);
        }
        if max_demand != f64::MIN {
            grouped_telemetry.set_max_demand(max_demand);
        }

        group_demands.push(grouped_telemetry);
    }

    group_demands
}

fn insert_demand_hist (grouped_demands: Vec<EnergyDemandTelemetry>, electric_circuit_id: i32, globs: &Arc<GlobalVars>) {
    for group in grouped_demands {
        let average_demand = group.demanda_med_at;
        let max_demand = group.max_demand;
        let min_demand = group.min_demand;
        let history = energy_demand_minutes_hist::EnergyDemandMinutesHist {
            electric_circuit_id,
            average_demand: Decimal::from_f64(average_demand).unwrap_or(Decimal::new(0, 0)),
            max_demand: Decimal::from_f64(max_demand).unwrap_or(Decimal::new(0, 0)),
            min_demand: Decimal::from_f64(min_demand).unwrap_or(Decimal::new(0, 0)),
            record_date: group.timestamp,
        };

        insert_data_demand(history, globs);
    }
}

async fn verify_telemetries_saved_data(parameters: &mut EnergyHistParams, start_date: NaiveDate, end_date: NaiveDate, electric_circuit_id: i32, globs: &Arc<GlobalVars>) -> Result<(), Box <dyn Error>> {
    parameters.start_time = NaiveDateTime::parse_from_str(&format!("{}T00:00:00", start_date), "%Y-%m-%dT%H:%M:%S").unwrap();

    let end_date_formatted = end_date + Duration::days(1);
    parameters.end_time = NaiveDateTime::parse_from_str(&format!("{}T00:15:00", end_date_formatted), "%Y-%m-%dT%H:%M:%S").unwrap();

    let response: String = match task_queue_manager(CompilationRequest::EnergyQuery(parameters.to_owned()), globs).await {
        Ok(response) => response,
        Err(err) => {
            return Err(format!("Erro ao executar requisição, {}", err).into());
        }
    };

    let mut response_data = match serde_json::from_str::<EnergyHist>(&response) {
        Ok(resp) => resp,
        Err(err) => {
            return Err(format!("Erro ao desserealizar JSON: {}", err).into());
        },
    };

    let response_data_filtered: Vec<PadronizedEnergyTelemetry> = response_data.data.into_iter().filter(|t| t.en_at_tri.unwrap_or(-1.0) >= 0.0).collect();

    if response_data_filtered.is_empty() {
        return Err("Não foi possível encontrar telemetrias válidas".into());
    }
    
    response_data.data = response_data_filtered;

    let mut date_array = Vec::new();
    let mut current_date = start_date;

    while current_date <= end_date {
        date_array.push(current_date.format("%Y-%m-%d").to_string());
        current_date += Duration::days(1);
    }
    
    for day in date_array {
        let mut response_data_clone = response_data.clone();
        let start_of_day = NaiveDateTime::parse_from_str(&format!("{}T00:00:00", day), "%Y-%m-%dT%H:%M:%S").unwrap();
        let end_of_day = NaiveDateTime::parse_from_str(&format!("{}T00:15:00", day), "%Y-%m-%dT%H:%M:%S").unwrap().checked_add_signed(Duration::days(1)).unwrap();

        response_data_clone.data.retain(|x| {
            if let Some(timestamp) = x.timestamp {
                timestamp >= start_of_day && timestamp < end_of_day
            } else {
                false
            }
        });

        let mut first_non_zero_history: Option<energy_hist::EnergyHist> = None;

        let grouped_telemetries_demand = group_telemetries_by_15_minutes(response_data_clone.data.clone(), &day);
        let grouped_averages = calculate_group_telemetries(&grouped_telemetries_demand);
        insert_demand_hist(grouped_averages, electric_circuit_id, globs);
        
        let compiled_energy_data = compile_energy_data(&response_data_clone, &day);
        for hour_data in compiled_energy_data.hours {
            let total_cons = format!("{:.2}", &hour_data.total_measured);

            let history = energy_hist::EnergyHist {
                electric_circuit_id,
                consumption: if hour_data.is_valid_consumption { Decimal::from_str_exact(&total_cons).unwrap() } else { Decimal::from(0) }, // utilizando unwrap devido à sempre ao total_cons sempre ter valor(padrão sendo 0)
                record_date: NaiveDateTime::parse_from_str(&format!("{} {}:00:00", day, hour_data.hour), "%Y-%m-%d %H:%M:%S").unwrap_or_default(),
                is_measured_consumption: hour_data.is_measured_consumption,
                is_valid_consumption: hour_data.is_valid_consumption,
            };

            if history.is_valid_consumption && !history.is_measured_consumption && first_non_zero_history.is_none() {
                first_non_zero_history = Some(history.clone());
            }

            insert_data_energy(history.clone(), globs);

            calc_energy_consumption_forecast(history.electric_circuit_id.clone(), history.record_date.clone(), globs)
        }

        // caso tenha buracos entre os dados de saved_data
        let Some(actual_history) = first_non_zero_history else {
            continue;
        };

        let last_history = match get_last_valid_consumption(actual_history.electric_circuit_id, actual_history.record_date, globs) {
            Ok(hist) => hist,
            Err(err) => {
                eprintln!("Erro ao encontrar último registro de consumo de energia, {:?}", err);
                None
            }
        };

        if last_history.as_ref().is_some_and(|h| h.record_date != actual_history.record_date - Duration::hours(1)) {
            verify_update_energy_consumption_aux(parameters, &actual_history, last_history.unwrap(), electric_circuit_id, globs).await?;
        }
    }

    Ok(())
}

fn verify_hours_online(energy_hist: EnergyHist, client_minutes_to_check_offline: Option<i32>, dri_interval: i32, start_date: NaiveDateTime, device_code: &str, unit_id: i32, day: &str, globs: &Arc<GlobalVars>) {
    let mut v_timestamp: Vec<String> = Vec::new();
    let end_date =  start_date + Duration::days(1);
    for energy in energy_hist.data {
        if let Some(timestamp) = energy.timestamp {
            if timestamp < end_date {
                v_timestamp.push(timestamp.format("%Y-%m-%dT%H:%M:%S").to_string().clone());
            }
        }
    }

    let mut hours_online = 0.0;

    if let Some(minutes_to_check) = client_minutes_to_check_offline {
        hours_online = check_amount_minutes_offline(minutes_to_check, v_timestamp, &start_date.format("%Y-%m-%dT%H:%M:%S").to_string());

    } else {
        let minutes = dri_interval / 60;
        hours_online = check_amount_minutes_offline(minutes, v_timestamp, &start_date.format("%Y-%m-%dT%H:%M:%S").to_string());
    }

    insert_device_disponibility_hist(unit_id, Decimal::from_f64_retain(hours_online).unwrap_or(Decimal::new(0,0)), day, device_code, globs);
}

fn calc_energy_consumption_forecast(electric_circuit_id: i32, date: NaiveDateTime, globs: &Arc<GlobalVars>) {
    let consumption_forecast = get_energy_consumption_average(GetEnergyAverage {
        date: date.clone().format("%Y-%m-%d %H:%M:%S").to_string(),
        electCircuitId: electric_circuit_id.clone()
    }, globs);

    let mut date_forecast = date.clone();

    date_forecast = date_forecast + Duration::days(7);

    if consumption_forecast.is_ok() {
        let consumption = consumption_forecast.unwrap().consumption_average.clone();
        for _ in 0..5 {
            let _ = insert_data_energy_consumption_forecast(energy_consumption_forecast::EnergyConsumptionForecast { 
                electric_circuit_id: electric_circuit_id.clone(), 
                consumption_forecast: consumption, 
                record_date: date_forecast.clone()
            }, globs);

            date_forecast = date_forecast + Duration::days(7);

        }
    } else {
        for _ in 0..5 {
            let _ = insert_data_energy_consumption_forecast(energy_consumption_forecast::EnergyConsumptionForecast { 
                electric_circuit_id: electric_circuit_id.clone(), 
                consumption_forecast: Decimal::new(0, 0), 
                record_date: date_forecast.clone()
            }, globs);

            date_forecast = date_forecast + Duration::days(7);
        }
    }
}

fn calc_consumption_monthly_target(unit_id: i32, day: &str, energy_devices: Vec<EnergyDevice>, globs: &Arc<GlobalVars>) {
    let date_day = NaiveDate::from_str(day).unwrap();
    let next_day = date_day + Duration::days(1);
    let is_last_day_of_month = next_day.month() != date_day.month();

    if is_last_day_of_month {
        let has_previous_monthly_target = match monthly_target_exists_for_unit(unit_id, globs) {
            Ok(result) => result.monthly_target_count > 0,
            Err(_) => false,
        };

        if !has_previous_monthly_target {
            let params = ParamsGetTotalDaysConsumptionUnit {
                unit_id,
                start_date: format!("{}-{}-01", date_day.year(), date_day.month()),
                end_date: date_day.to_string(),
            };

            match get_total_days_unit_with_consumption(params, globs) {
                Ok(days_data) => {
                    if days_data.days_count >= 25 {
                        calculate_and_insert_monthly_target(unit_id, date_day, next_day, energy_devices, globs);
                    }
                }
                Err(err) => {
                    let error_msg = format!(
                        "Erro ao obter dias com consumo para a unidade: {}, no dia {}, {}",
                        unit_id, date_day.to_string(), err
                    );
                    eprintln!("{}", err);
                    write_to_log_file_thread(&error_msg, 0, "ERROR");
                }
            }
        } else {
            calculate_and_insert_monthly_target(unit_id, date_day, next_day, energy_devices, globs);
        }
    }
}

pub fn reprocess_energy_forecast_view(day: &str, globs: &Arc<GlobalVars>) {
    let start_date = NaiveDate::parse_from_str(day, "%Y-%m-%d").unwrap();
    let end_date = start_date.checked_add_signed(Duration::days(30)).unwrap_or(start_date);

    
    let start_date_str = start_date.format("%Y-%m-%d").to_string();
    let end_date_str = end_date.format("%Y-%m-%d").to_string();

    process_energy_forecast_view(&start_date_str, &end_date_str, globs);
}

fn calculate_and_insert_monthly_target(
    unit_id: i32,
    date_day: NaiveDate,
    next_day: NaiveDate,
    energy_devices: Vec<EnergyDevice>,
    globs: &Arc<GlobalVars>,
) {
    let electric_circuits: Vec<i32> = energy_devices.iter().map(|device| device.electric_circuit_id).collect();

    if !electric_circuits.is_empty() {
        let monthly_forecast = get_energy_consumption_target(
            GetEnergyTarget {
                startDate: format!("{}-{}-01 00:00:00", date_day.year(), date_day.month()),
                endDate: format!("{}-{}-01 23:00:00", date_day.year(), date_day.month()),
                electCircuitIds: electric_circuits,
            },
            globs,
        );

        match monthly_forecast {
            Ok(consumption_average) => {
                let _ = insert_data_energy_monthly_consumption_target(
                    energy_monthly_consumption_target::EnergyMonthlyConsumptionTarget {
                        unit_id,
                        consumption_target: consumption_average.consumption_target,
                        date_forecast: next_day.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap()),
                    },
                    globs,
                );
            }
            Err(err) => {
                let error_msg = format!(
                    "Erro ao calcular a meta mensal para a unidade: {}, no dia {}, {}",
                    unit_id, date_day.to_string(), err
                );
                eprintln!("{}", err);
                write_to_log_file_thread(&error_msg, 0, "ERROR");
            }
        };
    }
}
