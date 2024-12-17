use serde::{Deserialize, Serialize};

use crate::telemetry_payloads::dri_telemetry::{DriCCNTelemetry, DriVAVandFancoilTelemetry};

use super::{common_func::{calcular_tempo_online, check_amount_minutes_offline}, compiler_common::SingleVariableCompilerFloat};

#[derive(Serialize, Deserialize, Debug)]
pub struct DRICCNTelemetryCompiler {
  pub tel_interval: Option<isize>,
  #[serde(rename="lastIndex")]
  pub last_index: isize,
  #[serde(rename="vTemp")]
  pub v_temp: SingleVariableCompilerFloat,
  pub v_timestamp: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DRICCNCompiledPeriod {
  pub hoursOnline: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DRIVAVandFancoilCompiledPeriod {
  pub hoursOnline: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DRIVAVandFancoilTelemetryCompiler {
  pub tel_interval: Option<isize>,
  #[serde(rename="lastIndex")]
  pub last_index: isize,
  #[serde(rename="vSetpoint")]
  pub v_setpoint: SingleVariableCompilerFloat,
  pub v_timestamp: Vec<String>,
}


impl DRICCNTelemetryCompiler {
    pub fn new(tel_interval: Option<isize>) -> DRICCNTelemetryCompiler {
      DRICCNTelemetryCompiler{
        tel_interval,
        last_index: -1,
        v_temp: SingleVariableCompilerFloat::create(1, 1, 1.0),
        v_timestamp: Vec::new()
      }
    }

    pub fn AdcPontos (&mut self, telemetry: &DriCCNTelemetry, index: isize) {
        if index <= self.last_index {
          return;
        }
        self.last_index = index;
        let interval = match self.tel_interval {
          Some(v) if v < 300 => 300 + 10,
          Some(v) => (v * 2) + 10,
          None => 300 + 10,
        };
        self.v_temp.adc_ponto_float(index, telemetry.Setpoint.map(f64::from), interval);
        self.v_timestamp.push(telemetry.timestamp.clone());
      }
    
      pub fn CheckClosePeriod (
        &mut self,
        periodLength: isize,
        check_minutes_offline: Option<i32>,
        start_date: &str,
      ) -> Result<Option<String>, String> {
        if self.v_temp.had_error() {
          return Err("There was an error compiling the data".to_owned());
        }
    
        if self.v_temp.is_empty() {
          return Ok(None);
        }
    
        let vec_temp = self.v_temp.fechar_vetor_completo(periodLength);
    
        let hours_online = if let Some(minutes_to_check) = check_minutes_offline {
          let percentage_hours = check_amount_minutes_offline(minutes_to_check, self.v_timestamp.clone(), start_date);

          percentage_hours
      } else {
          let hours = calcular_tempo_online(&vec_temp);
          let percentage_hours = ((hours * 100.0) / 24.0 * 100.0).round() / 100.0;
          percentage_hours
      };
        
        let mut data = serde_json::json!({});
        data["hours_online"] = hours_online.to_string().into();

        
        Ok(Some(data.to_string()))
      }
}

impl DRIVAVandFancoilTelemetryCompiler {
    pub fn new (tel_interval: Option<isize>) -> DRIVAVandFancoilTelemetryCompiler {
      DRIVAVandFancoilTelemetryCompiler{
        tel_interval,
        last_index: -1,
        v_setpoint: SingleVariableCompilerFloat::create(10, 1, 1.0),
        v_timestamp: Vec::new(),
      }
    }
  
    pub fn AdcPontos (&mut self, telemetry: &DriVAVandFancoilTelemetry, index: isize) {
      if index <= self.last_index {
        return;
      }
      let interval = match self.tel_interval {
        Some(v) if v < 300 => 300 + 10,
        Some(v) => (v * 2) + 10,
        None => 300 + 10,
      };
      self.last_index = index;
  
      self.v_setpoint.adc_ponto_float(index, telemetry.Setpoint, interval);
    }
  
    pub fn CheckClosePeriod (
      &mut self,
      periodLength: isize,
      check_minutes_offline: Option<i32>,
      start_date: &str,
    ) -> Result<Option<String>, String> {
      if self.v_setpoint.had_error() {
        return Err("There was an error compiling the data".to_owned());
      }
  
      if self.v_setpoint.is_empty() {
        return Ok(None);
      }
  
      let vecSetpoint = self.v_setpoint.fechar_vetor_completo(periodLength);

      let hours_online = if let Some(minutes_to_check) = check_minutes_offline {
        let percentage_hours = check_amount_minutes_offline(minutes_to_check, self.v_timestamp.clone(), start_date);

        percentage_hours
    } else {
        let hours = calcular_tempo_online(&vecSetpoint);
        let percentage_hours = ((hours * 100.0) / 24.0 * 100.0).round() / 100.0;
        percentage_hours
    };
        
      let mut data = serde_json::json!({});
      data["hours_online"] = hours_online.to_string().into();

      Ok(Some(data.to_string()))
    }
  }
