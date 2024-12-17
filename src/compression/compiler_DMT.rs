use serde::{Deserialize, Serialize};

use crate::telemetry_payloads::telemetry_formats::TelemetryDMT;

use super::{common_func::{calcular_tempo_online, check_amount_minutes_offline}, compiler_common::SingleVariableCompiler};

#[derive(Serialize, Deserialize, Debug)]
pub struct DMTTelemetryCompiler {
    #[serde(rename = "lastIndex")]
    pub last_index: isize,

    #[serde(rename = "vF1")]
    pub v_f1: SingleVariableCompiler,
    pub v_timestamp: Vec<String>,
}

impl DMTTelemetryCompiler {
    pub fn new(period_length: i64) -> DMTTelemetryCompiler {
        return DMTTelemetryCompiler {
            last_index: -1,
            v_f1: SingleVariableCompiler::create(),
            v_timestamp: Vec::new(),
        };
    }

    pub fn AdcPontos(&mut self, telemetry: &TelemetryDMT, index: isize) {
        if index <= self.last_index {
            return;
        }
        self.last_index = index;
        self.v_f1.adc_ponto(
            index,
            match telemetry.F1 {
                Some(false) => "0",
                Some(true) => "1",
                None => "",
            },
            150,
        );
        self.v_timestamp.push(telemetry.timestamp.clone());
    }

    pub fn CheckClosePeriod(
        &mut self,
        periodLength: isize,
        check_minutes_offline: Option<i32>,
        start_date: &str,
    ) -> Result<Option<CompiledPeriod>, String> {
        if self.v_f1.had_error() {
            return Err("There was an error compiling the data".to_owned());
        }
        let _periodData: Option<CompiledPeriod> = None;

        if self.v_f1.is_empty() {
            return Ok(None);
        }

        let vecF1 = self.v_f1.fechar_vetor_completo(periodLength);


        let hours_online = if let Some(minutes_to_check) = check_minutes_offline {
            let percentage_hours = check_amount_minutes_offline(minutes_to_check, self.v_timestamp.clone(), start_date);

            percentage_hours
        } else {
            let hours_online = calcular_tempo_online(&vecF1);
            let percentage_hours = ((hours_online * 100.0) / 24.0 * 100.0).round() / 100.0;
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
