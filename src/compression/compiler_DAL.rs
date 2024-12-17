use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::telemetry_payloads::telemetry_formats::TelemetryDAL;

use super::{common_func::{calcular_tempo_online, check_amount_minutes_offline}, compiler_common::SingleVariableCompiler};

#[derive(Serialize, Deserialize, Debug)]
pub struct DALTelemetryCompiler {
    #[serde(rename = "lastIndex")]
    pub last_index: isize,
    pub v_timestamp: Vec<String>,
    #[serde(rename = "vMode")]
    pub v_mode: Vec<SingleVariableCompiler>,
    #[serde(rename = "vRelays")]
    pub v_relays: Vec<SingleVariableCompiler>,
}

impl DALTelemetryCompiler {
    pub fn new(period_length: i64) -> DALTelemetryCompiler {
        return DALTelemetryCompiler {
            last_index: -1,
            v_timestamp: Vec::new(),
            v_mode: Vec::new(),
            v_relays: Vec::new(),
        };
    }

    pub fn AdcPontos (&mut self, telemetry: &TelemetryDAL, index: isize) {
      if index <= self.last_index {
        return;
      }
      self.last_index = index;
      for (mIndex, mValue) in telemetry.Mode.clone().into_iter().enumerate() {
        if self.v_mode.get(mIndex).is_none() {
          self.v_mode.push(SingleVariableCompiler::create())
        }
        let position = self.v_mode.get_mut(mIndex).unwrap();
        position.adc_ponto(index, &mValue, 90);
      }
      for (rIndex, rValue) in telemetry.Relays.clone().into_iter().enumerate() {
        if self.v_relays.get(rIndex).is_none() {
          self.v_relays.push(SingleVariableCompiler::create())
        }
        let position = self.v_relays.get_mut(rIndex).unwrap();
        position.adc_ponto(index, match rValue { Some(false) => "0", Some(true) => "1", None => "" }, 90);
      }
      self.v_timestamp.push(telemetry.timestamp.clone());
    }

    pub fn CheckClosePeriod(
        &mut self,
        periodLength: isize,
        check_minutes_offline: Option<i32>,
        start_date: &str,
    ) -> Result<Option<CompiledPeriod>, String> {
        let mut error = false;
        let mut empty = false;
        self.v_relays.iter().for_each(|x| {
            if x.had_error() {
                error = true;
            }
            if x.is_empty() {
                empty = true;
            }
        });
        if error {
            return Err("There was an error compiling the data".to_owned());
        }
        if empty {
            return Ok(None);
        }

        let mut vecMode = Vec::new();
        let mut vecTimestamp: Vec<String> = Vec::new();
        for mut mValue in self.v_mode.iter_mut() {
            let vecValue = mValue.fechar_vetor_completo(periodLength);
            vecMode.push(vecValue);
        }

        let modeTelemetries = if vecMode.len() > 0 { &vecMode[0] } else { "" };
        
        let hours_online = if let Some(minutes_to_check) = check_minutes_offline {
            let percentage_hours = check_amount_minutes_offline(minutes_to_check, self.v_timestamp.clone(), start_date);

            percentage_hours
        } else {
            let hours = calcular_tempo_online(modeTelemetries);
            let percentage_hours = ((hours * 100.0) / 24.0 * 100.0).round() / 100.0;
            percentage_hours
        };

        return Ok(Some(CompiledPeriod {
            hours_online,
        }));
    }
}

pub struct CompiledPeriod {
    pub hours_online: f64,
}
