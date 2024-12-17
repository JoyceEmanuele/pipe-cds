use std::error::Error;
use std::sync::Arc;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use crate::compression::compiler_DMA::DMATelemetryCompiler;
use crate::telemetry_payloads::dma_payload_json::get_raw_telemetry_pack_dma;
use crate::db::config::dynamo::QuerierDevIdTimestamp;
use crate::telemetry_payloads::dma_telemetry::{ split_pack };
use crate::GlobalVars;
use std::collections::HashMap;
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DmaHistParams {
    pub device_code: String,
    pub liters_per_pulse: Option<i32>,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub params: Option<Vec<String>>,
}


pub async fn process_comp_command_dma (rpars: ReqParameters, globs: &Arc<GlobalVars>) -> Result<String, Box<dyn Error>> {
    let rpars_serialized = serde_json::to_string(&rpars).unwrap();
    let dev_id = rpars.dev_id;
    let interval_length_s = rpars.interval_length_s;
    let ts_ini = rpars.ts_ini;
    let i_ts_ini = rpars.i_ts_ini;
    let ts_end = rpars.ts_end;
    let i_ts_end = rpars.i_ts_end;
    let check_minutes_offline: Option<i32> = rpars.check_minutes_offline;

  
    let accs: DmaData = {
        let page_ts_ini = ts_ini.clone();
        let tcomp = DMATelemetryCompiler::new(rpars.interval_length_s);
        DmaData {
          rpars: None,
          page_ts_ini,
          tcomp,
          telemetryList: [].to_vec(),
          timeOfTheLastTelemetry: "".to_string(),
        }
    };

    let page_ts_ini = accs.page_ts_ini;
    let mut tcomp = accs.tcomp;
  
    let mut table_name = {
      if (dev_id.len() == 12) && dev_id.to_uppercase().starts_with("DMA") {
        format!("{}XXXX_RAW", &dev_id.to_uppercase()[0..8])
      } else {
        String::new()
      }
    };
    
    for custom in &globs.configfile.CUSTOM_TABLE_NAMES_DMA {
      if dev_id.to_uppercase().starts_with(&custom.dev_prefix) {
        table_name = custom.table_name.to_owned();
        break;
      }
    }
  
    if table_name.len() == 0 {
        println!("Unknown DMA generation: {}", dev_id);
        return Ok("{}".to_string());
    }
  
    let querier = QuerierDevIdTimestamp::new_diel_dev(table_name, dev_id.clone());
  
    let mut found_invalid_payload = false;
    let mut is_first_of_the_day: bool = true;
  
    let start_day_query = NaiveDateTime::from_timestamp(i_ts_ini + 900, 0).format("%d").to_string();
    let end_day_query = NaiveDateTime::from_timestamp(i_ts_end - 900, 0).format("%d").to_string();
    found_invalid_payload = false;
    let mut last_number_of_pulses: Option<i32> = None;
    let mut pulsesPerHour:HashMap<String, i32> = HashMap::new();
    let mut lastTelemetryTime: String = "".to_string();
    let mut pulse_data_vector: Vec<PulseData> = Vec::new();

    let result = querier.run(&ts_ini, &ts_end, &mut |items| {
      for i in 1..items.len() {
        let mut payload = match get_raw_telemetry_pack_dma(&items[i]) {
          Ok(v) => v,
          Err(err) => {
            if !found_invalid_payload {
              // println!("Ignoring invalid payload(s): {}", &err);
            }
            found_invalid_payload = true;
            continue;
          },
        };
  
        let result = split_pack(&payload, i_ts_ini+900, i_ts_end,  &mut |telemetry, index| {
          tcomp.AdcPontos(telemetry, index);
        });
  
        match result {
          Ok(()) => {},
          Err(err) => {
            if !found_invalid_payload {
              // println!("Ignoring invalid payload(s): {}\n{:?}", &err, items[i].to_string());
            }
            found_invalid_payload = true;
            continue;
          },
        };
  
        let payload_pulses = match payload.pulses {
          Some(v) => v,
          None => {
            // Ignorar telemetria sem valor
            continue;
          },
        };
  
        if (start_day_query == payload.timestamp[8..10] || end_day_query == payload.timestamp[8..10]) && last_number_of_pulses.is_some() {
          let hour_str = &payload.timestamp[11..13];
          let hour: u32 = hour_str.parse().unwrap();
  
          let hour_str = format!("{:02}", hour); 
  
          lastTelemetryTime = payload.timestamp.clone();
    
          let new_pulse_data = PulseData {
            pulses: Some(payload.pulses.unwrap_or(0) as f64),
            timestamp: Some(NaiveDateTime::parse_from_str(&payload.timestamp, "%Y-%m-%dT%H:%M:%S").unwrap_or_default()),
          };
         pulse_data_vector.push(new_pulse_data);
          
          pulsesPerHour.entry(format!("{}:00", hour_str)).and_modify(|data| *data +=  payload.pulses.unwrap_or(0)).or_insert(payload.pulses.unwrap_or(0));
  
          is_first_of_the_day = false;
  
        }
  
  
        last_number_of_pulses = Some(payload_pulses);
      }
      return Ok(());
  
    }).await;
  
    let mut provision_error = false;
    if let Err(err) = result {
      if err.starts_with("ProvisionedThroughputExceeded:") {
        provision_error = true;
      } else if err.starts_with("ResourceNotFound:") {
        // println!("Table not found for: {}", dev_id);
        return Ok("{}".to_string());
      } else {
        return Ok(format!("ERROR[117] {}", err).to_string());
      }
    }
  
  
    let labels = vec!["00:00", "01:00", "02:00", "03:00", "04:00", "05:00", "06:00", "07:00", "08:00", "09:00", 
    "10:00", "11:00", "12:00", "13:00", "14:00", "15:00", "16:00", "17:00", "18:00", "19:00", "20:00", "21:00", "22:00", "23:00"];
    let mut formattedPulses:Vec<TelemetryPerTime> = Vec::new();
  
    for label in labels {
      formattedPulses.push( TelemetryPerTime { time: label.to_string(), pulses: match pulsesPerHour.get(&label.to_string()) {
        Some(v) => *v as i32,
        None => 0,
      } });
    }
  
  
    let mut dma_query_data = DmaData {
      rpars: Some(serde_json::from_str(&rpars_serialized).unwrap()),
      page_ts_ini,
      tcomp,
      timeOfTheLastTelemetry: lastTelemetryTime.to_string(),
      telemetryList: formattedPulses.clone()
    };

  
    let period_data = match dma_query_data.tcomp.CheckClosePeriod( isize::try_from(interval_length_s).unwrap(), check_minutes_offline, &ts_ini) {
        Err(err) => { println!("{}", err); return Ok("ERROR[120] CheckClosePeriod".to_string()); },
        Ok(v) => match v {
          Some(v) => v,
          None => {
            return Ok("{}".to_string());
          },
        }
      };
  
    let data = serde_json::json!({
      "TelemetryList": formattedPulses,
      "timeOfTheLastTelemetry": lastTelemetryTime,
      "provision_error": provision_error,
      "hours_online": period_data.hours_online,
      "data": pulse_data_vector
    });
  
    return Ok(data.to_string());
  }
  

  pub fn parse_parameters (dev_id: &str, day: &str, check_minutes_offline: Option<i32>) -> Result<ReqParameters, Box<dyn Error>> {
    if dev_id.len() < 9 {
      return Err(format!("Dma Hist -> ERROR! dev_id: {} length < 9", dev_id).into());
    }
  
    let interval_length_s = 24 * 60 * 60;
  
    let mut ts_ini = day;
  
    let mut i_ts_ini = match NaiveDateTime::parse_from_str(&format!("{}T00:00:00", ts_ini), "%Y-%m-%dT%H:%M:%S") {
      Err(err) => { println!("{} {}", &format!("{}T00:00:00", ts_ini), err); return Err(format!("Dma Hist -> ERROR! Parsing Date").into()); },
      Ok (mut date) => {
        date.timestamp()
      }
    };
  
    let ts_ini_aux = NaiveDateTime::from_timestamp(i_ts_ini, 0).format("%Y-%m-%dT%H:%M:%S").to_string();
    ts_ini = &ts_ini_aux;
    
    i_ts_ini -= 900;
    let new_ts_ini = NaiveDateTime::from_timestamp(i_ts_ini, 0).format("%Y-%m-%dT%H:%M:%S").to_string();
    let i_ts_end = i_ts_ini + interval_length_s + 900;
    let ts_end = NaiveDateTime::from_timestamp(i_ts_end, 0).format("%Y-%m-%dT%H:%M:%S").to_string();
  
  
    return Ok(ReqParameters {
      dev_id: dev_id.to_string(),
      interval_length_s,
      ts_ini: new_ts_ini,
      i_ts_ini,
      i_ts_end,
      ts_end,
      check_minutes_offline,
    });
  }
 

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReqParameters {
  pub dev_id: String,
  pub interval_length_s: i64,
  pub ts_ini: String,
  pub i_ts_ini: i64,
  pub i_ts_end: i64,
  pub ts_end: String,
  pub check_minutes_offline: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PulseData {
  pub pulses: Option<f64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub timestamp: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TelemetryPerTime {
  pub time: String,
  pub pulses: i32,
}

#[derive(Serialize, Deserialize)]
struct DmaData {
  pub rpars: Option<ReqParameters>,
  pub timeOfTheLastTelemetry: String,
  pub telemetryList: Vec<TelemetryPerTime>,
  pub page_ts_ini: String,
  pub tcomp: DMATelemetryCompiler,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DmaCompiledDayData {
  #[serde(rename = "TelemetryList")]
  pub telemetry_list: Vec<TelemetryPerTime>,
  #[serde(rename = "timeOfTheLastTelemetry")]
  pub time_of_the_last_telemetry: String,
  pub provision_error: bool,
  pub hours_online: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DmaCompiledData {
  #[serde(rename = "TelemetryList")]
  pub telemetry_list: Vec<TelemetryPerTime>,
  #[serde(rename = "timeOfTheLastTelemetry")]
  pub time_of_the_last_telemetry: String,
  pub provision_error: bool,
  pub hours_online: f64,
  pub data: Vec<PulseData>,
}

impl DmaCompiledData {
  fn new(telemetry_list: Vec<TelemetryPerTime>, time_of_the_last_telemetry: String, provision_error: bool, hours_online: f64, data: Vec<PulseData> ) -> Self {
      Self {
        telemetry_list,
        time_of_the_last_telemetry,
        provision_error,
        hours_online,
        data,
      }
  }

  pub fn num_fields_with_value<T: Serialize>(obj: &T) -> usize {
      let fields = serde_json::to_value(obj).unwrap();
      fields.as_object().unwrap().values().filter(|v| *v != &Value::Null).count()
  }
}

#[derive(Debug)]
pub struct DmaDataStruct {
    pub param: String,
    pub day: String,
    pub total_measured: f64,
    pub hour_values: HashMap<String, Vec<f64>>,
    pub hours: Vec<String>,
}

impl DmaDataStruct {
    pub fn new(day_consumption: &str) -> Self {
        Self {
            param: "pulses".to_string(),
            day: day_consumption.to_string(),
            total_measured: 0.0,
            hour_values: HashMap::new(),
            hours: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompiledDmaData {
    pub day: String,
    pub hours: Vec<HoursCompiledDmaData>,
    pub total_measured: f64,
}

#[derive(Debug, Clone)]
pub struct HoursCompiledDmaData {
    pub hour: String,
    pub total_measured: f64,
    pub last_pulses_hour: f64,
    pub first_pulses_hour: f64,
    pub is_measured_consumption: bool,
    pub is_valid_consumption: bool
}

impl CompiledDmaData {
    pub fn new(data_struct: &DmaDataStruct) -> Self {
        Self {
            day: data_struct.day.clone(),
            total_measured: 0.0,
            hours: Vec::new(),
        }
    }
}

