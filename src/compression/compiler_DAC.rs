use serde::{Deserialize, Serialize};

use std::str::FromStr;

use crate::telemetry_payloads::{dac_telemetry::HwInfoDAC, telemetry_formats::{TelemetryDAC_v3, TelemetryDAC_v3_calcs}};

use super::{common_func::check_amount_minutes_offline, compiler_common::{SingleVariableCompiler, SingleVariableCompilerBuilder, SingleVariableCompilerFloat}};


#[derive(Serialize, Deserialize, Debug)]
pub struct DACTelemetryCompiler {
    #[serde(rename = "lastIndex")]
    pub last_index: isize,
    #[serde(rename = "vLcmp")]
    pub v_lcmp: SingleVariableCompiler,
    pub v_timestamp: Vec<String>,
}

impl DACTelemetryCompiler {
  pub fn new(period_length: i64, cfg: &HwInfoDAC) -> DACTelemetryCompiler {
    let min_run = if cfg.isVrf || cfg.simulate_l1 {
      60isize
    }
    else {
      1isize
    };
    return DACTelemetryCompiler{
      last_index: -1,
      v_lcmp: SingleVariableCompilerBuilder::new().with_min_run_length(min_run).build_common(),
      v_timestamp: Vec::new(),
    };
  }

  pub fn AdcPontos (&mut self, telemetry: &TelemetryDAC_v3, index: isize, tolerance_time: i64) {
    if index <= self.last_index {
      return;
    }

    let sampling_time = if telemetry.saved_data == Some(true) {
      if tolerance_time < 15 { 15 } else { tolerance_time }
    } else {
        15
    } as isize;

    self.last_index = index;
    self.v_lcmp.adc_ponto(index, match telemetry.Lcmp { Some(false) => "0", Some(true) => "1", None => "" }, sampling_time);
    self.v_timestamp.push(telemetry.timestamp.clone());
  }

  pub fn CheckClosePeriod (
    &mut self,
    periodLength: isize,
    check_minutes_offline: Option<i32>,
    start_date: &str,
  ) -> Result<Option<CompiledPeriod>, String> {
    if self.v_lcmp.had_error() {
      return Err("There was an error compiling the data".to_owned());
    }
    let _periodData: Option<CompiledPeriod> = None;

    if self.v_lcmp.is_empty() {
      return Ok(None);
    }

    let mut hours_dev_on = 0.0;

    let vecLcmp = self.v_lcmp.fechar_vetor_completo(periodLength);
    let (hours_on, hours_off) = CalcularEstatisticasUso(&vecLcmp);

    if let Some(minutes_to_check) = check_minutes_offline {
      let formatted_date = start_date.replace(" ", "T");
      hours_dev_on = check_amount_minutes_offline(minutes_to_check, self.v_timestamp.clone(), &formatted_date);
    } else {
      let hours = hours_on + hours_off;
      let percentage_hours = ((hours * 100.0) / 24.0 * 100.0).round() / 100.0;
      hours_dev_on = percentage_hours;
    }

    return Ok(Some(CompiledPeriod{
      hours_on,
      hours_dev_on,
      vec_l1: vecLcmp,
      last_telemetry_time: self.v_timestamp.last().unwrap().to_string(),
    }));
  }
}

fn CalcularEstatisticasUso(vecL1: &str) -> (f64, f64) {
  let mut hoursOn = 0.0;
  let mut hoursOff = 0.0;
  if vecL1.is_empty() {
    return (hoursOn, hoursOff);
  }
  let imax = usize::try_from(vecL1.len() - 1).unwrap();
  let mut i: usize = 0;
  let mut ival: i64 = -1;
  let mut iast: i64 = -1;
  let mut value;
  let mut lastValue = "";
  let mut acc_duration: isize = 0;
  loop {
      if ival < 0 { ival = i as i64; }
      if i > imax || vecL1.as_bytes()[i] == b',' {
          let duration: isize;
          if iast < 0 {
              value = &vecL1[(ival as usize)..i];
              duration = 1;
          } else {
              value = &vecL1[(ival as usize)..(iast as usize)];
              duration = isize::from_str(&vecL1[((iast + 1) as usize)..i]).unwrap();
          }
          if value == "1" {
              hoursOn += (duration as f64) / 3600.0;
          }
          if value == "0" {
              hoursOff += (duration as f64) / 3600.0;
          }
          if !value.is_empty() { lastValue = value };
          ival = -1;
          iast = -1;
      }
      else if vecL1.as_bytes()[i] == b'*' { iast = i as i64; }
      if i > imax { break; }
      i += 1;
  }
  return (hoursOn, hoursOff);
}

pub struct CompiledPeriod {
  pub hours_on: f64,
  pub hours_dev_on: f64,
  pub vec_l1: String,
  pub last_telemetry_time: String,
}