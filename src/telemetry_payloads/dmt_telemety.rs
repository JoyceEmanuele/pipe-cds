use chrono::NaiveDateTime;

use super::telemetry_formats::{TelemetryDMT, TelemetryPackDMT};

pub fn split_pack(
    payload: &TelemetryPackDMT,
    ts_ini: i64,
    ts_next: i64,
    itemCallback: &mut dyn FnMut(&mut TelemetryDMT, isize),
) -> Result<(), String> {
    let pack_ts = match NaiveDateTime::parse_from_str(&payload.timestamp, "%Y-%m-%dT%H:%M:%S") {
        Err(_) => {
            println!("Error parsing Date:\n{:?}", payload);
            return Err("Error parsing Date".to_owned());
        }
        Ok(date) => date.timestamp(),
    };

    let mut telemetry = TelemetryDMT {
        timestamp: payload.timestamp.to_owned(),
        dev_id: payload.dev_id.to_owned(),
        F1: None,
    };

    let pack_ts = match NaiveDateTime::parse_from_str(&payload.timestamp, "%Y-%m-%dT%H:%M:%S") {
        Err(_) => {
            println!("Error parsing Date:\n{:?}", payload);
            return Err("Error parsing Date".to_owned());
        }
        Ok(date) => date.timestamp(),
    };

    checkSetTelemetryValues(payload, &mut telemetry);

    if (pack_ts >= ts_ini) && (pack_ts < ts_next) {
        itemCallback(&mut telemetry, isize::try_from(pack_ts - ts_ini).unwrap());
    }
    return Ok(());
}

fn checkSetTelemetryValues(payload: &TelemetryPackDMT, telemetry: &mut TelemetryDMT) {
    telemetry.F1 = if payload.Feedback.len() >= 4 {
        payload.Feedback[0]
    } else {
        None
    };
}
