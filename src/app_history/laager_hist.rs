use serde_json::Value;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};


#[derive(Debug)]
pub struct LaagerDataStruct {
    pub param: String,
    pub day: String,
    pub total_measured: f64,
    pub hour_values: HashMap<String, Vec<f64>>,
    pub hours: Vec<String>,
}

impl LaagerDataStruct {
    pub fn new(day_consumption: &str) -> Self {
        Self {
            param: "usage".to_string(),
            day: day_consumption.to_string(),
            total_measured: 0.0,
            hour_values: HashMap::new(),
            hours: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompiledLaagerData {
    pub day: String,
    pub hours: Vec<HoursCompiledLaagerData>,
    pub total_measured: f64,
}

#[derive(Debug, Clone)]
pub struct HoursCompiledLaagerData {
    pub hour: String,
    pub total_measured: f64,
    pub last_usage_hour: f64,
    pub first_usage_hour: f64,
    pub is_measured_consumption: bool,
    pub is_valid_consumption: bool
}

impl CompiledLaagerData {
    pub fn new(data_struct: &LaagerDataStruct) -> Self {
        Self {
            day: data_struct.day.clone(),
            total_measured: 0.0,
            hours: Vec::new(),
        }
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct LaagerConsumptionHistoryPerHour {
    pub date: String,
    pub usage: f64,
    pub reading: Option<f64>,
    #[serde(rename = "readings_per_day")]
    pub data: Vec<ReadingPerDayLaager>
}

impl LaagerConsumptionHistoryPerHour {
    fn new( date: String, usage: f64, reading: Option<f64>, data: Vec<ReadingPerDayLaager> ) -> Self {
        Self {
            date,
            usage,
            reading,
            data,
        }
    }
    pub fn num_fields_with_value<T: Serialize>(obj: &T) -> usize {
        let fields = serde_json::to_value(obj).unwrap();
        fields.as_object().unwrap().values().filter(|v| *v != &Value::Null).count()
    }
  }
  #[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ReadingPerDayLaager {
    pub time: String,
    pub reading: Option<f64>,
    pub usage: Option<f64>
}

#[derive(Debug, Deserialize)]
pub struct LaagerConsumption {
    pub history: Vec<LaagerConsumptionHistoryPerHour>,
}