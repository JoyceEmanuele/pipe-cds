use crate::telemetry_payloads::telemetry_formats::TelemetryPackDMT;
use crate::telemetry_payloads::parse_json_props::{
    get_string_prop,
    get_int_number_prop,
    get_bool_array_prop
};


pub fn get_raw_telemetry_pack_dmt(item: &serde_json::Value) -> Result<TelemetryPackDMT,String> {
    let telemetry = TelemetryPackDMT {
        timestamp: match get_string_prop(&item.get("timestamp")) {
            Ok(timestamp) => timestamp,
            Err(message) => { println!("Invalid timestamp"); return Err(message); }
        },
        dev_id: match get_string_prop(&item.get("dev_id")) {
          Ok(dev_id) => dev_id,
          Err(message) => { println!("Invalid dev_id"); return Err(message); }
        },
        samplingTime: get_int_number_prop(&item.get("samplingTime")).unwrap_or(1), // de quantos em quantos segundos o firmware lê os sensores e insere nos vetores.
        Feedback: match get_bool_array_prop(&item.get("Feedback")) {
            Ok(Feedback) => Feedback,
            Err(message) => { 
                // println!("Invalid Feedback: {}", message); 
                return Err(message); }
        }
    };

    return Ok(telemetry);
}
