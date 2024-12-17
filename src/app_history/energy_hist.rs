use std::{collections::HashMap, sync::Arc, error::Error};

use chrono::{Duration, NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::db::config::dynamo::QuerierDevIdTimestamp;

use crate::models::external_models::device::EnergyDevice;
use crate::telemetry_payloads::energy::dme::TelemetryDME;
use crate::telemetry_payloads::energy::padronized::{format_padronized_energy_temeletry, PadronizedEnergyTelemetry};
use crate::GlobalVars;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnergyHistParams {
    pub energy_device_id: String,
    pub serial: String,
    pub manufacturer: String,
    pub model: String,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub formulas: Option<HashMap<String, String>>,
    pub params: Option<Vec<String>>,
}

impl EnergyHistParams {
    pub async fn process_query(mut self, globs: &Arc<GlobalVars>) -> Result<String, Box<dyn Error>> {
        let tels = match &self.manufacturer[..] {
            "Diel Energia" => self.process_dme_query(globs).await,
            _ => Err("Unknown manufacturer!".into()),
        }?;

        let data = serde_json::json!({
            "energy_device_id": self.energy_device_id,
            "serial": self.serial,
            "manufacturer": self.manufacturer,
            "model": self.model,
            "data": tels
          });

          Ok(data.to_string())
    }

    async fn process_dme_query(&self, globs: &Arc<GlobalVars>) -> Result<Vec<PadronizedEnergyTelemetry>, Box<dyn Error>> {
        let dev_id_upper = self.energy_device_id.to_uppercase();
        let mut table_name = {
            if (self.energy_device_id.len() == 12) && dev_id_upper.starts_with("DRI") {
                format!("{}XXXX_RAW", &dev_id_upper[0..8])
            } else {
                String::new()
            }
        };

        for custom in &globs.configfile.CUSTOM_TABLE_NAMES_DRI {
            if dev_id_upper.starts_with(&custom.dev_prefix) {
                table_name = custom.table_name.to_owned();
                break;
            }
        }

        if table_name.is_empty() {
            return Err(format!("Unknown DRI generation: {}", self.energy_device_id).into());
        }

        let ts_ini = self.start_time.format("%Y-%m-%dT%H:%M:%S").to_string();
        let ts_end = self.end_time.format("%Y-%m-%dT%H:%M:%S").to_string();
        let querier = QuerierDevIdTimestamp::new_custom(table_name.to_owned(), "dev_id".to_owned(), "timestamp".to_owned(), self.energy_device_id.to_owned());
        let mut final_tels = Vec::new();

        querier.run(&ts_ini, &ts_end, &mut |tels: Vec<TelemetryDME>| {
            let mut x = tels.into_iter()
                .filter_map(|mut tel| {
                    tel.formulas = self.formulas.clone();
                    tel.try_into().ok()
                })
                .collect::<Vec<PadronizedEnergyTelemetry>>();
            final_tels.append(&mut x);
            Ok(())
        }).await?;

        let formatted_final_tels = final_tels.into_iter().map(|tel| format_padronized_energy_temeletry(tel, self.params.as_ref())).collect::<Vec<PadronizedEnergyTelemetry>>();

        Ok(formatted_final_tels)
    }

    pub fn parse_parameters(energy_device: &EnergyDevice, day: &str) -> EnergyHistParams {
        let end_date = NaiveDate::parse_from_str(day, "%Y-%m-%d").unwrap() + Duration::days(1);
        let end_time = NaiveDateTime::parse_from_str(&format!("{}T00:15:00", end_date), "%Y-%m-%dT%H:%M:%S").unwrap();

        EnergyHistParams {
            energy_device_id: energy_device.device_code.clone(),
            serial: energy_device.serial.clone().unwrap_or_default(),
            manufacturer: energy_device.manufacturer.clone(),
            model: energy_device.model.clone().unwrap_or_default(),
            start_time: NaiveDateTime::parse_from_str(&format!("{}T00:00:00", day), "%Y-%m-%dT%H:%M:%S").unwrap(),
            end_time,
            formulas: energy_device.formulas.clone(),
            params: Some(["en_at_tri".to_string(), "demanda_med_at".to_string()].to_vec()),
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnergyHist {
    pub energy_device_id: String,
    serial: String,
    manufacturer: String,
    model: String,
    pub data: Vec<PadronizedEnergyTelemetry>,
}

impl EnergyHist {
    fn new(energy_device_id: String, serial: String, manufacturer: String, model: String, data: Vec<PadronizedEnergyTelemetry>) -> Self {
        Self {
            energy_device_id,
            serial,
            manufacturer,
            model,
            data,
        }
    }

    pub fn num_fields_with_value<T: Serialize>(obj: &T) -> usize {
        let fields = serde_json::to_value(obj).unwrap();
        fields.as_object().unwrap().values().filter(|v| *v != &Value::Null).count()
    }
}

#[derive(Debug)]
pub struct EnergyDataStruct {
    pub param: String,
    pub day: String,
    pub total_measured: f64,
    pub hour_values: HashMap<String, Vec<f64>>,
    pub hours: Vec<String>,
}

impl EnergyDataStruct {
    pub fn new(day_consumption: &str) -> Self {
        Self {
            param: "en_at_tri".to_string(),
            day: day_consumption.to_string(),
            total_measured: 0.0,
            hour_values: HashMap::new(),
            hours: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompiledEnergyData {
    pub day: String,
    pub hours: Vec<HoursCompiledEnergyData>,
    pub total_measured: f64,
}

#[derive(Debug, Clone)]
pub struct HoursCompiledEnergyData {
    pub hour: String,
    pub total_measured: f64,
    pub last_en_at_tri_hour: f64,
    pub first_en_at_tri_hour: f64,
    pub is_measured_consumption: bool,
    pub is_valid_consumption: bool
}

impl CompiledEnergyData {
    pub fn new(data_struct: &EnergyDataStruct) -> Self {
        Self {
            day: data_struct.day.clone(),
            total_measured: 0.0,
            hours: Vec::new(),
        }
    }
}