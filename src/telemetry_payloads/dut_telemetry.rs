use chrono::NaiveDateTime;

use super::{dut_l1::l1_calc::DutL1Calculator, telemetry_formats::{TelemetryDUT_v3, TelemetryPackDUT_v2}};


pub struct HwInfoDUT {
    pub temperature_offset: f64,
}

pub fn split_pack(
    payload: &TelemetryPackDUT_v2,
    ts_ini: i64,
    ts_next: i64,
    dut_state: &mut dyn DutL1Calculator,
    itemCallback: &mut dyn FnMut(&TelemetryDUT_v3, isize),
    dev: &HwInfoDUT,
) -> Result<(), String> {
    let pack_length = payload
        .Temperature.as_ref().map(|x| x.len())
        .or_else(|| payload.eCO2.as_ref().map(|x| x.len()))
        .or_else(|| payload.Temperature_1.as_ref().map(|x| x.len()))
        .unwrap_or(0);

    if let Some(Humidity) = &payload.Humidity {
        if Humidity.len() != pack_length { return Err("Incompatible length".to_owned()) }
    }
    if let Some(Temperature) = &payload.Temperature {
        if Temperature.len() != pack_length { return Err("Incompatible length".to_owned()) }
    }
    if let Some(Temperature_1) = &payload.Temperature_1 {
        if Temperature_1.len() != pack_length { return Err("Incompatible length".to_owned()) }
    }
    if let Some(raw_eCO2) = &payload.raw_eCO2 {
        if raw_eCO2.len() != pack_length { return Err("Incompatible length".to_owned()) }
    }
    if let Some(Tmp) = &payload.Tmp {
        if Tmp.len() != pack_length { return Err("Incompatible length".to_owned()) }
    }

    let pack_ts = payload.timestamp.timestamp();
    let sampling_time: i64 = payload.samplingTime; // de quantos em quantos segundos o firmware lê os sensores e insere nos vetores.

    let mut telemetry = TelemetryDUT_v3{
        timestamp: payload.timestamp,
        Temp: None,
        Temp1: None,
        Tmp: None,
        Hum: None,
        eCO2: None,
        raw_eCO2: None,
        tvoc: None,
        State: None,
        Mode: None,
        l1: None,
    };

    if pack_length == 0 {
        // payload is automation only
        checkSetAutTelemetryValues(payload, &mut telemetry);
        if (pack_ts >= ts_ini) && (pack_ts < ts_next) {
            itemCallback(&telemetry, isize::try_from(pack_ts - ts_ini).unwrap());
        }
    } else {
        let mut remainingSteps = pack_length;
        let mut telm_ts;
        for _i in 0..pack_length {
            telemetry.timestamp = NaiveDateTime::from_timestamp_opt(pack_ts - ((remainingSteps as i64 - 1) * sampling_time), 0).unwrap();
            remainingSteps = checkSetTelemetryValues(payload, &mut telemetry, dut_state, remainingSteps, dev);
            telm_ts = pack_ts - ((remainingSteps as i64) * sampling_time);
            if telm_ts < ts_ini {
                continue;
            }
            if telm_ts >= ts_next {
                continue;
            }
            telemetry.timestamp = NaiveDateTime::from_timestamp_opt(telm_ts, 0).unwrap();
            itemCallback(&telemetry, isize::try_from(telm_ts - ts_ini).unwrap());
        }
    }

    return Ok(());
}

fn checkSetAutTelemetryValues(payload: &TelemetryPackDUT_v2, telemetry: &mut TelemetryDUT_v3) {
    telemetry.Temp = None;
    telemetry.Temp1 = None;
    telemetry.Hum = None;
    telemetry.State = payload.State.as_ref().map(|v| v.to_owned());
    telemetry.Mode = payload.Mode.as_ref().map(|v| v.to_owned());
}

fn checkSetTelemetryValues(
    payload: &TelemetryPackDUT_v2,
    telemetry: &mut TelemetryDUT_v3,
    dut_l1_calc: &mut dyn DutL1Calculator,
    mut remainingSteps: usize,
    dev: &HwInfoDUT,
) -> usize {
    let offset_temp = dev.temperature_offset;
    let payload_length = payload
        .Temperature
        .as_ref()
        .map(|t| t.len())
        .or_else(|| payload.eCO2.as_ref().map(|e| e.len()))
        .or_else(|| payload.Temperature_1.as_ref().map(|e| e.len()))
        .expect("Não foi possível identificar o número de leituras no payload.");

    if remainingSteps == 0 {
        remainingSteps = payload_length - 1;
    } else {
        remainingSteps -= 1;
    }
    let index = payload_length - 1 - remainingSteps;

    telemetry.Temp = match &payload.Temperature {
        None => None,
        Some(Temperature) => match Temperature.get(index) {
            None => None,
            Some(Temp) => Temp.filter(|t| *t > -99.0 && *t < 85.0).map(|t| t + offset_temp),
        }
    };
    telemetry.Temp1 = match &payload.Temperature_1 {
        None => None,
        Some(Temperature_1) => match Temperature_1.get(index) {
            None => None,
            Some(Temp1) => Temp1.filter(|t| *t > -99.0 && *t < 85.0),
        }
    };
    telemetry.Tmp = match &payload.Tmp {
        None => None,
        Some(Tmp) => match Tmp.get(index) {
            None => None,
            Some(Tmp) => Tmp.filter(|t| *t > -99.0 && *t < 85.0),
        }
    };
    telemetry.Hum = match &payload.Humidity {
        None => None,
        Some(Humidity) => match Humidity.get(index) {
            None => None,
            Some(Hum) => Hum.filter(|h| *h >= 0.0),
        },
    };
    telemetry.eCO2 = match &payload.eCO2 {
        None => None,
        Some(eCO2) => match eCO2.get(index) {
            None => None,
            Some(eCO2) => eCO2.filter(|c| *c != -99),
        }
    };
    telemetry.raw_eCO2 = match &payload.raw_eCO2 {
        None => None,
        Some(raw_eCO2) => match raw_eCO2.get(index) {
            None => None,
            Some(raw_eCO2) => raw_eCO2.filter(|c| *c != -99),
        }
    };
    telemetry.tvoc = match &payload.tvoc {
        None => None,
        Some(tvoc) => match tvoc.get(index) {
            None => None,
            Some(tvoc) => tvoc.filter(|t| *t > -99),
        }
    };
    telemetry.State = payload.State.clone();
    telemetry.Mode = payload.Mode.as_ref().map(|mode| {
        if mode.as_str() == "AUTO" {
            "Auto".to_owned()
        } else {
            mode.to_owned()
        }
    });
    telemetry.l1 = dut_l1_calc.calc_l1_tel_v3(telemetry, payload.samplingTime, dev).ok().flatten();    

    return remainingSteps;
}
