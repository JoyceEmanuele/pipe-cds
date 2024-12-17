use chrono::NaiveDateTime;

use super::telemetry_formats::{TelemetryDAL, TelemetryPackDAL};

pub fn split_pack(
    payload: &TelemetryPackDAL,
    ts_ini: i64,
    ts_next: i64,
    itemCallback: &mut dyn FnMut(&mut TelemetryDAL, isize),
) -> Result<(), String> {
    let pack_ts = match NaiveDateTime::parse_from_str(&payload.timestamp, "%Y-%m-%dT%H:%M:%S") {
        Err(_) => {
            return Err("Error parsing Date".to_owned());
        }
        Ok(date) => date.timestamp(),
    };

    let mut telemetry = TelemetryDAL {
        timestamp: payload.timestamp.to_owned(),
        dev_id: payload.dev_id.to_owned(),
        State: payload.State.to_owned(),
        Mode: payload.Mode.to_owned(),
        Relays: payload.Relays.to_owned(),
        Feedback: payload.Feedback.to_owned(),
        gmt: payload.gmt.to_owned(),
    };

    let pack_ts = match NaiveDateTime::parse_from_str(&payload.timestamp, "%Y-%m-%dT%H:%M:%S") {
        Err(_) => {
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

fn checkSetTelemetryValues(payload: &TelemetryPackDAL, telemetry: &mut TelemetryDAL) {
    telemetry.Relays = if payload.Relays.len() >= 4 {payload.Relays.to_owned()} else { Vec::new() };
    telemetry.Feedback = if payload.Feedback.len() >= 4 {payload.Feedback.to_owned()} else { Vec::new() };
}
