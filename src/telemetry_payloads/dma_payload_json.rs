use crate::telemetry_payloads::telemetry_formats::TelemetryPackDMA;
use crate::telemetry_payloads::parse_json_props::{
    get_string_prop,
    get_int_number_prop,
};

pub fn get_raw_telemetry_pack_dma(item: &serde_json::Value) -> Result<TelemetryPackDMA, String> {
    let telemetry = TelemetryPackDMA {
        timestamp: match get_string_prop(&item.get("timestamp")) {
            Ok(v) => v,
            Err(message) => {
                println!("Invalid timestamp:\n{:?}", &item);
                return Err(format!("Invalid telemetry contents [217]: {}", message));
            }
        },
        pulses: match &item.get("pulses") {
            None => None,
            Some (prop) => match get_int_number_prop(&item.get("pulses")) {
                Ok(v) => Some(v as i32),
                Err(_message) => None,
            },
        },
        mode: match &item.get("mode") {
            None => None,
            Some (prop) => match get_string_prop(&item.get("mode")) {
                Ok(v) => Some(v),
                Err(_message) => None,
            },
        },
        operation_mode: match &item.get("operation_mode") {
            None => None,
            Some (prop) => match get_int_number_prop(&item.get("operation_mode")) {
                Ok(v) => Some(v as i16),
                Err(_message) => None,
            },
        },
        dev_id: match get_string_prop(&item.get("dev_id")) {
            Ok(dev_id) => dev_id,
            Err(message) => { println!("Invalid dev_id"); return Err(message); }
        },
        samplingTime: match &item.get("samplingTime") {
            None => None,
            Some (prop) => match get_int_number_prop(&item.get("samplingTime")) {
                Ok(v) => Some(v as i16),
                Err(_message) => None,
            },
        },
    };


    return Ok(telemetry);
}
