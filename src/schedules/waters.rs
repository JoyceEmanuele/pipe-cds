use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::error::Error;
use crate::app_history::dma_hist::{CompiledDmaData, DmaCompiledData, DmaDataStruct, HoursCompiledDmaData, PulseData};
use crate::app_history::laager_hist::{CompiledLaagerData, HoursCompiledLaagerData, LaagerConsumptionHistoryPerHour, LaagerDataStruct, ReadingPerDayLaager};
use crate::db::entities::water_consumption_forecast::insert_update_water_consumption_forecast;
use crate::models::database_models::water_consumption_forecast::WaterConsumptionForecast;
use crate::schedules::scheduler::write_to_log_file_thread;
use crate::GlobalVars;
use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Weekday};

use rust_decimal::Decimal;
use rust_decimal::prelude::*;

use crate::models::external_models::device::{DmaDevice, LaagerDevice, WaterConsumptionHistory};
use crate::models::database_models::{waters_hist, water_hist};

use crate::schedules::devices::{process_dmas_devices_per_hour, process_laager_devices_per_hour};

use crate::db::entities::waters_hist::{get_last_valid_consumption, get_water_consumption_in_dates, insert_data_water, insert_data_waters};

pub async fn process_waters_devices(unit_id: i32, laager_device: Option<LaagerDevice>, dma_device: Option<DmaDevice>, day: &str, client_minutes_to_check_offline: Option<i32>, globs: &Arc<GlobalVars>) {
  if let Some(device) = laager_device {
    process_laager_devices_per_hour(unit_id, day, &device, globs).await;
  }
  
  if let Some(device) = dma_device {
    process_dmas_devices_per_hour(unit_id, day, &device, client_minutes_to_check_offline, globs).await;
  }
} 

pub async fn verify_update_water_consumption(
    actual_history: &water_hist::WaterHist,
    device_code: &str,
    supplier: &str,
    installation_date: Option<&str>,
    unit_id: i32, globs: &Arc<GlobalVars>) -> Result <(), Box<dyn Error>>{
    let history = match get_last_valid_consumption(actual_history.unit_id, actual_history.record_date, globs) {
        Ok(hist) => hist,
        Err(err) => {
            eprintln!("Erro ao encontrar último registro de consumo de água, {:?}", err);
            None
        }
    };
    
    if history.as_ref().is_some_and(|h| h.record_date != actual_history.record_date - Duration::hours(1)) {
        let mut days = HashSet::new();
        let history_clone = history.unwrap().clone();
        let hours_without_consumption = (actual_history.record_date - history_clone.record_date).num_hours();
        let formatted_consumption = actual_history.consumption.to_f64().unwrap_or(0.0) / hours_without_consumption as f64;

        for i in 1..hours_without_consumption + 1 {
            let date_time = history_clone.record_date + Duration::hours(i.into());
            let day = date_time.date();
            days.insert(day);
            let is_last = i == hours_without_consumption;
            let history = water_hist::WaterHist {
                unit_id,
                supplier: supplier.to_string(),
                consumption: Decimal::from_f64_retain(formatted_consumption).unwrap_or(Decimal::from(0)).round_dp(3),
                device_code: String::from(device_code),
                record_date: date_time,
                is_measured_consumption: if is_last { false } else { true },
                is_valid_consumption: true,
            };

            insert_data_water(history, globs);
        }

        for day in days {
            get_consumption_forecast(day, installation_date, unit_id, globs);
        }   
    }
    else {
        let date_initial = actual_history.record_date.date();
        let time_initial = NaiveTime::from_hms(0, 0, 0);
        let initial_history = NaiveDateTime::new(date_initial, time_initial);
        let hours_without_consumption = (actual_history.record_date - initial_history).num_hours() + 1;
        let formatted_consumption = actual_history.consumption.to_f64().unwrap_or(0.0) / hours_without_consumption as f64;

        for i in 0..hours_without_consumption  {
            let date_time = initial_history + Duration::hours(i.into());
            let is_last = i == hours_without_consumption - 1;
            let history = water_hist::WaterHist {
                unit_id,
                supplier: supplier.to_string(),
                consumption: Decimal::from_f64_retain(formatted_consumption).unwrap_or(Decimal::from(0)).round_dp(3),
                device_code: String::from(device_code),
                record_date: date_time,
                is_measured_consumption: if is_last { false } else { true },
                is_valid_consumption: true,
            };

            insert_data_water(history, globs);
        }

        get_consumption_forecast(date_initial, installation_date, unit_id, globs);
    }


    Ok(())
}

pub fn compile_dma_data(dma_hist: &DmaCompiledData, day: &str) -> CompiledDmaData {
    let sample_with_params = dma_hist.data.iter().find(|&x| DmaCompiledData::num_fields_with_value(x) > 1);
    let mut data_struct = generate_data_struct_dma(day, sample_with_params);
    format_data_dma(&mut data_struct, dma_hist)
}

fn generate_data_struct_dma(day: &str, sample: Option<&PulseData>) -> DmaDataStruct {
    let mut result = DmaDataStruct::new(day);

    if sample.is_some() {
        for hour in 0..24 {
            let hour_str = format!("{:02}", hour);
            result.hour_values.insert(hour_str.clone(), Vec::new());
            result.hours.push(hour_str.clone());
        }
    }

    result
}

fn format_data_dma(data_struct: &mut DmaDataStruct, water_hist: &DmaCompiledData) -> CompiledDmaData {
    let data = &water_hist.data;

    for hist in data {
        if hist.pulses.is_some() {
            let (hour) = {
                let timestamp = hist.timestamp.unwrap();
                let hour = timestamp.time().hour();
                let hour_str = format!("{:02}", hour); 
                hour_str
            };

            data_struct.hour_values.get_mut(&hour[..2]).unwrap().push(hist.pulses.unwrap());
        } else {
            continue;
        }
    }

    let mut obj = CompiledDmaData::new(&data_struct);
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

        let hour_data_value = if hour_data_vec.len() > 0 {
            hour_data_vec[hour_data_vec.len() - 1] - hour_data_vec.first().copied().unwrap_or(0.0)
        } else {
            hour_data_vec.first().copied().unwrap_or(0.0)
        };

        let mut last_pulses = hour_data_vec.last().copied().unwrap_or(0.0);

        let next_hour_value = if let Some(next_data) = &next_hour_data_vec {
            if next_data.len() > 0 {
                last_pulses = next_data[0];
                next_data[0] - hour_data_vec.first().copied().unwrap_or(0.0)
            } else {
                hour_data_value
            }
        } else {
            hour_data_value
        };

        obj.hours.push(HoursCompiledDmaData{
            hour: hour.to_string(),
            total_measured: if !hour_data_vec.is_empty() && next_hour_value >= 0.0  { next_hour_value } else { 0.0 },
            last_pulses_hour: last_pulses,
            first_pulses_hour: hour_data_vec.first().copied().unwrap_or(0.0),
            is_measured_consumption: false,
            is_valid_consumption: if !hour_data_vec.is_empty() && next_hour_value >= 0.0  { true } else { false }
        });

        let mut total_measured = 0.0;

        for hour_data in &obj.hours {
            total_measured += hour_data.total_measured;
        }
        
        obj.total_measured = total_measured;
    }

    let mut formatedWaterData = obj.clone();

    while currentPosition < obj.hours.len() {
        if obj.hours[currentPosition].total_measured == 0.0 && !obj.hours[currentPosition].is_valid_consumption {
            
            let mut position = currentPosition;
            while position < obj.hours.len() && obj.hours[position].total_measured == 0.0 {
                if last_non_zero_index.is_some() && last_non_zero_index.unwrap() < currentPosition {
                    let _ = zeroPositions.push(position);
                }
                position += 1;
            }
            currentPosition = position;
        } else {
            if let Some(last_index) = last_non_zero_index {
                if !zeroPositions.is_empty() {
                    let last_pulses = obj.hours[last_index].last_pulses_hour;
                    let next_first_pulses = obj.hours[currentPosition].first_pulses_hour;
                    
                    let total_consumption = next_first_pulses - last_pulses;
                    let calculated_total_measured = total_consumption / (zeroPositions.len() as f64);

                    for &i in &zeroPositions {
                        formatedWaterData.hours[i].total_measured = calculated_total_measured;
                        formatedWaterData.hours[i].is_measured_consumption = true;
                        formatedWaterData.hours[i].is_valid_consumption = true;
                    }
    
                    zeroPositions.clear();
                }
            }
            last_non_zero_index = Some(currentPosition);
            currentPosition += 1;
        }
    }

    formatedWaterData
}

pub async fn insert_data_dma_per_hour(
    device_code: &str,
    liters_per_pulse: i32,
    supplier: &str,
    unit_id: i32,
    water_data: &CompiledDmaData,
    installation_date: Option<&str>,
    globs: &Arc<GlobalVars>) {
    let mut first_non_zero_history: Option<water_hist::WaterHist> = None;
    let mut consumption_sum = Decimal::new(0,0);

    for hour in &water_data.hours {
        let total_cons = &hour.total_measured * liters_per_pulse as f64;
        let total_cons_formated = format!("{:.2}", total_cons);
        consumption_sum += Decimal::from_str(&total_cons_formated).unwrap_or(Decimal::new(0, 0));

        let history = water_hist::WaterHist {
            unit_id,
            supplier: supplier.to_string(),
            consumption: Decimal::from_str_exact(&total_cons_formated).unwrap(),
            device_code: String::from(device_code),
            record_date: NaiveDateTime::parse_from_str(&format!("{} {}:00:00", water_data.day, hour.hour), "%Y-%m-%d %H:%M:%S").unwrap_or_default(),
            is_measured_consumption: hour.is_measured_consumption,
            is_valid_consumption: hour.is_valid_consumption,
        };

        if history.is_valid_consumption && !history.is_measured_consumption && first_non_zero_history.is_none() {
            first_non_zero_history = Some(history.clone());
        }

        
        insert_data_water(history, globs);
    }
    
    if let Some(water_installation_date) = installation_date {
        let day = water_data.day.as_str();
        if water_installation_date <= day {
            verify_last_three_weeks_consumption(NaiveDate::from_str(day).unwrap_or_default(), water_installation_date, unit_id, consumption_sum, globs);
        }
    }
    
    if first_non_zero_history.is_some() {
        match verify_update_water_consumption(&first_non_zero_history.unwrap(), 
         device_code, supplier, installation_date, unit_id, globs).await {
            Ok(res) => { },
            Err(err) => {eprintln!("Não foi possível verificar o último consumo válido: {:?}", err)}
        };
    }
}

pub fn normalize_laager_consumption(history: &mut Vec<LaagerConsumptionHistoryPerHour>) {
    let mut first_null_index: Option<usize> = None;
    for day_index in 0..history.len() {
        if (history[day_index].reading.is_none() && first_null_index.is_none()) {
            first_null_index = Some(day_index);
        } else if history[day_index].reading.is_some() {
            if (first_null_index.is_some_and(|x| x > 0)) {
                let last_reading_before_null = history[first_null_index.unwrap() - 1].reading;
                let quantity_of_null_days = day_index - first_null_index.unwrap() + 1;
                let interval_reading = history[day_index].reading.unwrap() - last_reading_before_null.unwrap();
                let estimated_consumption_by_day = (interval_reading * 1000.0) / quantity_of_null_days as f64;

                for update_day_index in first_null_index.unwrap()..=day_index {
                    history[update_day_index].usage = estimated_consumption_by_day;
                }
            }
        }
        first_null_index = None;
    }
}

pub fn compile_laager_data(laager_hist: &LaagerConsumptionHistoryPerHour, day: &str) -> CompiledLaagerData {
    let sample_with_params = laager_hist.data.iter().find(|&x| LaagerConsumptionHistoryPerHour::num_fields_with_value(x) > 1);
    let mut data_struct = generate_data_struct_laager(day, sample_with_params);
    format_data_laager(&mut data_struct, laager_hist)
}

fn generate_data_struct_laager(day: &str, sample: Option<&ReadingPerDayLaager>) -> LaagerDataStruct {
    let mut result = LaagerDataStruct::new(day);

    if sample.is_some() {
        for hour in 0..24 {
            let hour_str = format!("{:02}", hour);
            result.hour_values.insert(hour_str.clone(), Vec::new());
            result.hours.push(hour_str.clone());
        }
    }

    result
}

fn format_data_laager(data_struct: &mut LaagerDataStruct, laager_hist: &LaagerConsumptionHistoryPerHour) -> CompiledLaagerData {
    let data = &laager_hist.data;

    for hist in data {
        if hist.usage.is_some() {
            let (hour) = {                
                let hour_str = format!("{:02}", hist.time); 
                hour_str
            };

            data_struct.hour_values.get_mut(&hour[..2]).unwrap().push(hist.usage.unwrap());
        } else {
            continue;
        }
    }

    let mut obj = CompiledLaagerData::new(&data_struct);
    let mut hours_sorted: Vec<&String> = data_struct.hour_values.keys().collect();
    hours_sorted.sort_by(|a, b| a.cmp(b));
    let mut zeroPositions: Vec<usize> = Vec::new();
    let mut currentPosition = 0;
    let mut last_non_zero_index: Option<usize> = None;

    for (hour_index, hour) in hours_sorted.iter().enumerate() {
        let hour_data_vec = data_struct.hour_values.get(&hour.to_string()).unwrap();
        let mut next_hour_data_vec: Option<Vec<f64>> = 
        if *hour != "23" {
            data_struct.hour_values.get(&format!("{:02}", hour_index + 1)).cloned()
        } else {
            None
        };

        let sum: f64 = hour_data_vec.iter().sum();
        let is_sum_invalid: bool = if hour_data_vec.len() > 1 {
            sum <= 0.0 && hour_data_vec.iter().any(|&x| x != 0.0)
        } else {
            sum < 0.0
        };

         obj.hours.push(HoursCompiledLaagerData{
            hour: hour.to_string(),
            total_measured: if !hour_data_vec.is_empty() && !is_sum_invalid { sum } else { 0.0 },
            last_usage_hour: 0.0,
            first_usage_hour: 0.0,
            is_measured_consumption: false,
            is_valid_consumption: if !hour_data_vec.is_empty() && !is_sum_invalid { true } else { false }
        });

        let mut total_measured = 0.0;

        for hour_data in &obj.hours {
            total_measured += hour_data.total_measured;
        }
        
        obj.total_measured = total_measured;
    }

    let mut formatedWaterData = obj.clone();

    while currentPosition < obj.hours.len() {
        if !obj.hours[currentPosition].is_valid_consumption {
            
            let mut position = currentPosition;
            while position < obj.hours.len() && obj.hours[position].total_measured == 0.0 && !obj.hours[position].is_valid_consumption {
                if last_non_zero_index.is_some() && last_non_zero_index.unwrap() < currentPosition {
                    let _ = zeroPositions.push(position);
                }
                position += 1;
            }
            currentPosition = position;
        } else {
            if let Some(last_index) = last_non_zero_index {
                if !zeroPositions.is_empty() {
                    let _ = zeroPositions.push(currentPosition);
                    
                    let total_consumption = obj.hours[currentPosition].total_measured;
                    let calculated_total_measured = total_consumption / (zeroPositions.len() as f64);

                    for (pos, &i) in zeroPositions.iter().enumerate() {
                        formatedWaterData.hours[i].total_measured = calculated_total_measured;
                        if pos == zeroPositions.len() - 1 {
                            formatedWaterData.hours[i].is_measured_consumption = false;
                        } else {
                            formatedWaterData.hours[i].is_measured_consumption = true;
                        }
                        formatedWaterData.hours[i].is_valid_consumption = true;
                    }
    
                    zeroPositions.clear();
                }
            }
            last_non_zero_index = Some(currentPosition);
            currentPosition += 1;
        }
    }

    formatedWaterData
}

pub async fn insert_data_laager_per_hour(
    device_code: &str,
    supplier: &str,
    unit_id: i32,
    water_data: &CompiledLaagerData,
    installation_date: Option<&str>,
    globs: &Arc<GlobalVars>) {
    let mut first_non_zero_history: Option<water_hist::WaterHist> = None;
    let mut consumption_sum = Decimal::new(0, 0);
    
    for hour in &water_data.hours {
        let total_cons = format!("{:.2}", &hour.total_measured);
        consumption_sum += Decimal::from_str(&total_cons).unwrap_or(Decimal::new(0, 0));

        let history = water_hist::WaterHist {
            unit_id,
            supplier: supplier.to_string(),
            consumption: Decimal::from_str_exact(&total_cons).unwrap(),
            device_code: String::from(device_code),
            record_date: NaiveDateTime::parse_from_str(&format!("{} {}:00:00", water_data.day, hour.hour), "%Y-%m-%d %H:%M:%S").unwrap_or_default(),
            is_measured_consumption: hour.is_measured_consumption,
            is_valid_consumption: hour.is_valid_consumption,
        };

        if history.is_valid_consumption && !history.is_measured_consumption && first_non_zero_history.is_none() {
            first_non_zero_history = Some(history.clone());
        }

        insert_data_water(history, globs);
    }

    if let Some(water_installation_date) = installation_date {
        let day = water_data.day.as_str();
        if water_installation_date <= day {
            verify_last_three_weeks_consumption(NaiveDate::from_str(day).unwrap_or_default(), water_installation_date, unit_id, consumption_sum, globs);
        }
    }

    if first_non_zero_history.is_some() {
        match verify_update_water_consumption(&first_non_zero_history.unwrap(), device_code, supplier, installation_date, unit_id, globs).await {
            Ok(res) => { },
            Err(err) => {eprintln!("Não foi possível verificar o último consumo válido: {:?}", err)}
        };
    }
}

fn get_consumption_forecast(forecast_date: NaiveDate, installation_date: Option<&str>, unit_id: i32, globs: &Arc<GlobalVars>) {
    if let Some(water_installation_date) = installation_date.clone() {
        if water_installation_date <= forecast_date.to_string().as_str() {
            let actual_consumption = match get_water_consumption_in_dates(unit_id, [forecast_date].to_vec(), globs) {
                Ok(res) => res.consumption,
                Err(err) => {
                    let error_msg = format!("Error when obtaining consumption, {:?}", err);
                    println!("{}", error_msg);
                    write_to_log_file_thread(&error_msg, 0, "ERROR");
                    return;
                }
            };

            verify_last_three_weeks_consumption(forecast_date, water_installation_date, unit_id, actual_consumption, globs);
        }
    }
}

fn get_last_three_days_weeks(day: NaiveDate) -> Vec<NaiveDate> {
    let mut days = Vec::new();
    let mut date = day;

    for _ in 0..3 {
        date = date - Duration::weeks(1);
        days.push(date);
    }

    days
} 

pub fn forecast_without_consumption_by_day(installation_date: Option<String>, day: &str, unit_id: i32, globs: &Arc<GlobalVars>) {
    if let Some(water_installation_date) = installation_date.clone() {
        if water_installation_date <= day.to_string() {
            verify_last_three_weeks_consumption(NaiveDate::from_str(day).unwrap_or_default(), &water_installation_date, unit_id, Decimal::new(0, 0), globs);
        }
    }
}

fn verify_last_three_weeks_consumption(forecast_date: NaiveDate, installation_date: &str, unit_id: i32, actual_consumption: Decimal, globs: &Arc<GlobalVars>) {
    let days = get_last_three_days_weeks(forecast_date);
    let installation_date_aux = NaiveDate::parse_from_str(&installation_date, "%Y/%m/%d")
    .map_err(|e| {
        write_to_log_file_thread(&format!("Error on unit {:?} parsing installation_date: {:?}, {:?}", unit_id, installation_date, e), 0, "ERROR");
        eprintln!("Error parsing installation_date: {}", e);
        e
    }).ok();

    if let Some(date) = installation_date_aux {
        let days_after_installation_date = days.iter()
            .filter(|&&day| day >= date);
    
        let qtd_days_after_installation_date = days_after_installation_date.clone().count() as i64;
    
        let consumption = match get_water_consumption_in_dates(unit_id, days_after_installation_date.cloned().collect(), globs) {
            Ok(res) => res.consumption,
            Err(err) => {
                let error_msg = format!("Error when obtaining consumption, {:?}", err);
                println!("{}", error_msg);
                write_to_log_file_thread(&error_msg, 0, "ERROR");
                return;
            }
        };
    
        let weekday = forecast_date.weekday();
    
        verify_forecast_usage(weekday, unit_id, forecast_date, consumption, actual_consumption, qtd_days_after_installation_date, globs);
    }
}

pub fn verify_forecast_usage(
    weekday: Weekday,
    unit_id: i32,
    date: NaiveDate,
    sum_consumption: Decimal,
    consumption_today: Decimal,
    qtd_days_after_installation_date: i64,
    globs: &Arc<GlobalVars>,
) {
    let forecast_usage = (sum_consumption + consumption_today)/Decimal::new(qtd_days_after_installation_date + 1, 0);
    let (monday, tuesday, wednesday, thursday, friday, saturday, sunday) = match weekday {
        Weekday::Mon => (
            Some(forecast_usage),
            None, None, None, None, None, None
        ),
        Weekday::Tue => (
            None,
            Some(forecast_usage),
            None, None, None, None, None
        ),
        Weekday::Wed => (
            None, None,
            Some(forecast_usage),
            None, None, None, None
        ),
        Weekday::Thu => (
            None, None, None,
            Some(forecast_usage),
            None, None, None
        ),
        Weekday::Fri => (
            None, None, None, None,
            Some(forecast_usage),
            None, None
        ),
        Weekday::Sat => (
            None, None, None, None, None,
            Some(forecast_usage),
            None
        ),
        Weekday::Sun => (
            None, None, None, None, None, None,
            Some(forecast_usage)
        ),
    };

    let forecast = WaterConsumptionForecast {
        unit_id,
        forecast_date: date.with_day(1).unwrap_or_default(),
        monday,
        tuesday,
        wednesday,
        thursday,
        friday,
        saturday,
        sunday,
    };

    insert_update_water_consumption_forecast(forecast, globs);
}
