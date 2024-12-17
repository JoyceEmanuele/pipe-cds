use crate::compression::compiler_common::{SingleVariableCompiler, SingleVariableCompilerFloat};
use crate::telemetry_payloads::telemetry_formats::{TelemetryDMA};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use super::common_func::{calcular_tempo_online, check_amount_minutes_offline};

#[derive(Serialize, Deserialize, Debug)]
pub struct DMATelemetryCompiler {
    #[serde(rename = "lastIndex")]
    pub last_index: isize,

    #[serde(rename = "vPulses")]
    pub v_pulses: SingleVariableCompilerFloat,
    #[serde(rename = "vMode")]
    pub v_mode: SingleVariableCompiler,
    pub v_timestamp: Vec<String>,
}

impl DMATelemetryCompiler {
  pub fn new(period_length: i64) -> DMATelemetryCompiler {
    return DMATelemetryCompiler{
      last_index: -1,
      v_pulses: SingleVariableCompilerFloat::create(5, 1, 0.2),
      v_mode: SingleVariableCompiler::create(),
      v_timestamp: Vec::new(),
    };
  }

  pub fn AdcPontos (&mut self, telemetry: &TelemetryDMA, index: isize) {
    if index <= self.last_index {
      return;
    }

    let tolerance_time = isize::try_from(match telemetry.samplingTime { Some(v) => v*2 + 20, None => 60 }).unwrap();

    self.last_index = index;
    self.v_pulses.adc_ponto_float(index, match telemetry.pulses { Some(v) => Some(f64::from(v)), None => None }, tolerance_time);
    self.v_mode.adc_ponto(index,  match &telemetry.mode { Some(v) => v, None => "" }, tolerance_time);
    self.v_timestamp.push(telemetry.timestamp.clone());
  }

  pub fn CheckClosePeriod (
    &mut self,
    periodLength: isize,
    check_minutes_offline: Option<i32>,
    start_date: &str,
  ) -> Result<Option<CompiledPeriod>, String> {
    if self.v_pulses.had_error() {
      return Err("There was an error compiling the data".to_owned());
    }
    let _periodData: Option<CompiledPeriod> = None;

    if self.v_pulses.is_empty() {
      return Ok(None);
    }

    let vecPulses = self.v_pulses.fechar_vetor_completo(periodLength);
    let vecMode = self.v_mode.fechar_vetor_completo(periodLength);

    let hours_online = if let Some(minutes_to_check) = check_minutes_offline {
      let percentage_hours = check_amount_minutes_offline(minutes_to_check, self.v_timestamp.clone(), start_date);

      percentage_hours
  } else {
      let hours_online = calcular_tempo_online(&vecMode);
      let percentage_hours = ((hours_online * 100.0) / 24.0 * 100.0).round() / 100.0;
      percentage_hours
  };

    return Ok(Some(CompiledPeriod{
      hours_online,
      Pulses: vecPulses,
    }));
  }
}

pub struct CompiledPeriod {
  pub hours_online: f64,
  pub Pulses: String,
}
