use crate::telemetry_payloads::telemetry_formats::TelemetryPackDUT_v2;

use super::parse_json_props::get_string_prop;

pub fn get_raw_telemetry_pack_dut(item: &serde_json::Value) -> Result<TelemetryPackDUT_v2, String> {
    let telemetry = match serde_json::from_value::<TelemetryPackDUT_v2>(item.clone()) {
        Ok(x) => x,
        Err(message) => {
            // println!("{message}");
            return Err(message.to_string());
        }
    };

    let mut array_length = telemetry.Temperature.as_ref().and_then(|v| Some(v.len()));
    if array_length.is_none() {
        array_length = telemetry.Humidity.as_ref().and_then(|v| Some(v.len()));
    }
    if array_length.is_none() {
        array_length = telemetry.eCO2.as_ref().and_then(|v| Some(v.len()));
    }
    if array_length.is_none() {
        array_length = telemetry.tvoc.as_ref().and_then(|v| Some(v.len()));
    }

    let array_length = match array_length {
        Some(v) => v,
        None => {
            if let Err(message) = get_string_prop(&item.get("Mode")) {
                // println!("Invalid telemetry contents:\n{:?}", &item);
                return Err("Invalid telemetry contents [219]".to_owned());
            }
            0
        }
    };

    return Ok(telemetry);
}