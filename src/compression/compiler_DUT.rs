use serde::{Deserialize, Serialize};
use std::str::FromStr;

use super::compiler_common::{
    SingleVariableCompiler, SingleVariableCompilerBuilder, SingleVariableCompilerFloat,
};

use crate::{compression::common_func::check_amount_minutes_offline, telemetry_payloads::telemetry_formats::TelemetryDUT_v3};

#[derive(Serialize, Deserialize, Debug)]
pub struct DUTTelemetryCompiler {
    #[serde(rename = "lastIndex")]
    pub last_index: isize,
    #[serde(rename = "vTemp")]
    pub v_temp: SingleVariableCompilerFloat,
    #[serde(rename = "vHum")]
    pub v_hum: SingleVariableCompilerFloat,
    #[serde(rename = "vCO2")]
    pub v_eco2: SingleVariableCompilerFloat,
    #[serde(rename = "vL1")]
    pub v_l1: SingleVariableCompiler,
    pub v_timestamp: Vec<String>,
}

impl DUTTelemetryCompiler {
    pub fn new() -> DUTTelemetryCompiler {
        return DUTTelemetryCompiler {
            last_index: -1,
            v_temp: SingleVariableCompilerFloat::create(5, 1, 0.2),
            v_hum: SingleVariableCompilerFloat::create(5, 1, 0.2),
            v_eco2: SingleVariableCompilerFloat::create(1, 25, 0.2),
            v_l1: SingleVariableCompilerBuilder::new()
                .with_min_run_length(5)
                .build_common(),
            v_timestamp: Vec::new(),
        };
    }

    pub fn AdcPontos(&mut self, telemetry: &TelemetryDUT_v3, index: isize) {
        if index <= self.last_index {
            return;
        }
        self.last_index = index;
        let l1 = match telemetry.l1 {
            Some(true) => "1",
            Some(false) => "0",
            None => "",
        };
        self.v_temp.adc_ponto_float(index, telemetry.Temp, 180);
        self.v_hum.adc_ponto_float(index, telemetry.Hum, 180);
        self.v_eco2
            .adc_ponto_float(index, telemetry.eCO2.map(f64::from), 180);

        self.v_l1.adc_ponto(index, l1, 180);
        self.v_timestamp.push(telemetry.timestamp.format("%Y-%m-%dT%H:%M:%S").to_string().clone());
    }

    pub fn CheckClosePeriod(
        &mut self,
        periodLength: isize,
        check_minutes_offline: Option<i32>,
        start_date: &str,
    ) -> Result<Option<CompiledPeriod>, String> {
        if self.v_temp.had_error() || self.v_eco2.had_error() {
            return Err("There was an error compiling the data".to_owned());
        }

        if self.v_temp.is_empty() && self.v_eco2.is_empty() {
            return Ok(None);
        }

        let vecTemp = self.v_temp.fechar_vetor_completo(periodLength);
        let vecHum = self.v_hum.fechar_vetor_completo(periodLength);
        let vec_l1 = self.v_l1.fechar_vetor_completo(periodLength);

        let hasL1 = &vec_l1 != "*86400";
        let mut hours_online = 0.0;
        let mut hours_on_l1 = 0.0;

        if hasL1 {
            let (hoursOnlineL1, hoursOfflineL1) = calcular_estatisticas_dut_duo(&vec_l1);
            hours_on_l1 = hoursOnlineL1;

            if let Some(minutes_to_check) = check_minutes_offline {
                hours_online = check_amount_minutes_offline(minutes_to_check, self.v_timestamp.clone(), start_date);
            } else {
                let hours = hoursOnlineL1 + hoursOfflineL1;
                let percentage_hours = ((hours * 100.0) / 24.0 * 100.0).round() / 100.0;
                hours_online = percentage_hours;
            };
        } else {
            if let Some(minutes_to_check) = check_minutes_offline {
                hours_online = check_amount_minutes_offline(minutes_to_check, self.v_timestamp.clone(), start_date);
            } else {
                let hours = calcular_tempo_online(&vecTemp, &vecHum);
                let percentage_hours = ((hours * 100.0) / 24.0 * 100.0).round() / 100.0;
                hours_online = percentage_hours;
            };
        }

        return Ok(Some(CompiledPeriod {
            hours_online,
            hours_on_l1,
            vec_l1,
        }));
    }
}

fn calcular_estatisticas_dut_duo(vecL1: &str) -> (f64, f64) {
    // "*200,3.5*300,2,2*30,*100"
    let mut hoursOnline = 0.0;
    let mut hoursOffline = 0.0;

    if vecL1.is_empty() {
        return (hoursOnline, hoursOffline);
    }
    let imax = usize::try_from(vecL1.len() - 1).unwrap();
    let mut i: usize = 0;
    let mut ival: i64 = -1;
    let mut iast: i64 = -1;
    let mut value;
    let mut lastValue = "";

    loop {
        if ival < 0 {
            ival = i as i64;
        }
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
                hoursOnline += (duration as f64) / 3600.0;
            }
            if value == "0" {
                hoursOffline += (duration as f64) / 3600.0;
            }
            if !value.is_empty() {
                lastValue = value
            };
            ival = -1;
            iast = -1;
        } else if vecL1.as_bytes()[i] == b'*' {
            iast = i as i64;
        }
        if i > imax {
            break;
        }
        i += 1;
    }
    return (hoursOnline, hoursOffline);
}

fn calcular_tempo_online(vecTemp: &str, vecHum: &str) -> (f64) {
    let mut hoursOnline = 0.0;

    if vecTemp.is_empty() && vecHum.is_empty() {
        return hoursOnline;
    }

    if !vecTemp.is_empty() {
        let imax = usize::try_from(vecTemp.len() - 1).unwrap();
        let mut i: usize = 0;
        let mut ival: i64 = -1;
        let mut iast: i64 = -1;
        let mut value;
        loop {
            if ival < 0 {
                ival = i as i64;
            }
            if i > imax || vecTemp.as_bytes()[i] == b',' {
                let duration: isize;
                if iast < 0 {
                    value = &vecTemp[(ival as usize)..i];
                    duration = 1;
                } else {
                    value = &vecTemp[(ival as usize)..(iast as usize)];
                    duration = isize::from_str(&vecTemp[((iast + 1) as usize)..i]).unwrap();
                }
                if !value.is_empty() {
                    hoursOnline += (duration as f64) / 3600.0;
                }
                ival = -1;
                iast = -1;
            } else if vecTemp.as_bytes()[i] == b'*' {
                iast = i as i64;
            }
            if i > imax {
                break;
            }
            i += 1;
        }
    }

    if !vecHum.is_empty() && hoursOnline == 0. {
        let imax = usize::try_from(vecHum.len() - 1).unwrap();
        let mut i: usize = 0;
        let mut ival: i64 = -1;
        let mut iast: i64 = -1;
        let mut value;
        loop {
            if ival < 0 {
                ival = i as i64;
            }
            if i > imax || vecHum.as_bytes()[i] == b',' {
                let duration: isize;
                if iast < 0 {
                    value = &vecHum[(ival as usize)..i];
                    duration = 1;
                } else {
                    value = &vecHum[(ival as usize)..(iast as usize)];
                    duration = isize::from_str(&vecHum[((iast + 1) as usize)..i]).unwrap();
                }
                if !value.is_empty() {
                    hoursOnline += (duration as f64) / 3600.0;
                }
                ival = -1;
                iast = -1;
            } else if vecHum.as_bytes()[i] == b'*' {
                iast = i as i64;
            }
            if i > imax {
                break;
            }
            i += 1;
        }
    }

    return hoursOnline;
}

pub struct CompiledPeriod {
    pub hours_online: f64,
    pub hours_on_l1: f64,
    pub vec_l1: String,
}
