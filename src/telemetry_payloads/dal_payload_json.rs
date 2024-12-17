use crate::telemetry_payloads::telemetry_formats::TelemetryPackDAL;
use crate::telemetry_payloads::parse_json_props::{
    get_string_prop,
    get_bool_array_prop,
    get_string_array_prop,
};

pub fn get_raw_telemetry_pack_dal(item: &serde_json::Value) -> Result<TelemetryPackDAL,String> {
    let telemetry = TelemetryPackDAL {
        timestamp: match get_string_prop(&item.get("timestamp")) {
            Ok(timestamp) => timestamp,
            Err(message) => { println!("Invalid timestamp: {:?} {}", &item, message); return Err(message); }
        },
        dev_id: match get_string_prop(&item.get("dev_id")) {
          Ok(dev_id) => dev_id,
          Err(message) => { println!("Invalid dev_id: {:?} {}", &item, message); return Err(message); }
        },
        State: match get_string_prop(&item.get("State")) {
          Ok(State) => State,
          Err(message) => { println!("Invalid State: {:?} {}", &item, message); return Err(message); }
        },
        Mode: match get_string_array_prop(&item.get("Mode")) {
          Ok(Mode) => Mode,
          Err(message) => { println!("Invalid Mode: {:?} {}", &item, message); return Err(message); }
        },
        Relays: match get_bool_array_prop(&item.get("Relays")) {
          Ok(Relays) => Relays,
          Err(message) => { println!("Invalid Relays: {:?} {}", &item, message); return Err(message); }
        },
        Feedback: match get_bool_array_prop(&item.get("Feedback")) {
            Ok(Feedback) => Feedback,
            Err(message) => { println!("Invalid Feedback: {:?} {}", &item, message); return Err(message); }
        },
        gmt: match get_string_prop(&item.get("gmt")) {
          Ok(gmt) => Some(gmt),
          Err(message) => { Some("-3".to_string()) }
      },
    };

    return Ok(telemetry);
}
