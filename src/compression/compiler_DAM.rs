use serde::{Deserialize, Serialize};

use crate::telemetry_payloads::telemetry_formats::TelemetryRawDAM_v1;

use super::{common_func::{calcular_tempo_online, check_amount_minutes_offline}, compiler_common::{SingleVariableCompiler, SingleVariableCompilerFloat}};


#[derive(Serialize, Deserialize, Debug)]
pub struct DAMTelemetryCompiler {
    pub lastIndex: isize,
    pub vState: SingleVariableCompiler,
    pub v_timestamp: Vec<String>,
}

impl DAMTelemetryCompiler {
    pub fn new() -> DAMTelemetryCompiler {
        return DAMTelemetryCompiler {
            lastIndex: -1,
            vState: SingleVariableCompiler::create(),
            v_timestamp: Vec::new(),
        };
    }

    pub fn AdcPontos(&mut self, telemetry: &TelemetryRawDAM_v1, index: isize) {
        if index <= self.lastIndex {
            return;
        }
        self.lastIndex = index;
        self.vState.adc_ponto(index, &telemetry.State, 150);
        self.v_timestamp.push(telemetry.timestamp.clone());
    }

    pub fn CheckClosePeriod(
        &mut self,
        periodLength: isize,
        check_minutes_offline: Option<i32>,
        start_date: &str,
    ) -> Result<Option<CompiledPeriod>, String> {
        if self.vState.had_error() {
            return Err("There was an error compiling the data".to_owned());
        }

        if self.vState.is_empty() {
            return Ok(None);
        }

        let vecState = self.vState.fechar_vetor_completo(periodLength);

        let hours_online = if let Some(minutes_to_check) = check_minutes_offline {
            let percentage_hours = check_amount_minutes_offline(minutes_to_check, self.v_timestamp.clone(), start_date);
      
            percentage_hours
        } else {
            let hours_online = calcular_tempo_online(&vecState);
            let percentage_hours = ((hours_online * 100.0) / 24.0 * 100.0).round() / 100.0;
            percentage_hours
        };

        Ok(Some(CompiledPeriod {
            hours_online,
        }))
    }
}

pub struct CompiledPeriod {
    pub hours_online: f64,
}
