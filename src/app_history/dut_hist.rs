use std::{error::Error, sync::Arc, time};
use chrono::{Duration, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{compression::compiler_DUT::DUTTelemetryCompiler, db::config::dynamo::QuerierDevIdTimestamp, telemetry_payloads::dut_payload_json::get_raw_telemetry_pack_dut, GlobalVars};
use crate::telemetry_payloads::dut_telemetry::{ split_pack, HwInfoDUT };
use crate::telemetry_payloads::dut_l1::l1_calc::create_l1_calculator;


#[derive(Serialize, Deserialize, Debug)]
pub struct ReqParameters {
  pub dev_id: String,
  pub interval_length_s: i64,
  pub ts_ini: String,
  pub i_ts_ini: i64,
  pub i_ts_end: i64,
  pub ts_end: String,
  pub offset_temp: f64,
  pub check_minutes_offline: Option<i32>,
}

pub async fn process_comp_command_dut (rpars: ReqParameters, globs: &Arc<GlobalVars>) -> Result<String, Box<dyn Error>> {
    let rpars_serialized = serde_json::to_string(&rpars).unwrap();
    let dev_id = rpars.dev_id;
    let interval_length_s = rpars.interval_length_s;
    let ts_ini = rpars.ts_ini;
    let i_ts_ini = rpars.i_ts_ini;
    let i_ts_end = rpars.i_ts_end;
    let ts_end = rpars.ts_end;
    let offset_temp = rpars.offset_temp;
    let check_minutes_offline = rpars.check_minutes_offline;
    let dev = HwInfoDUT {
        temperature_offset: offset_temp
      };

    let accs: DutData = {
        let page_ts_ini = ts_ini.clone();
        let tcomp = DUTTelemetryCompiler::new();
        DutData {
            rpars: None,
            page_ts_ini,
            tcomp
        }
    };

    let page_ts_ini = accs.page_ts_ini;
    let mut tcomp = accs.tcomp;
  
    let mut table_name = {
      if (dev_id.len() == 12) && dev_id.to_uppercase().starts_with("DUT") {
        format!("{}XXXX_RAW", &dev_id.to_uppercase()[0..8])
      } else {
        String::new()
      }
    };
  
    for custom in &globs.configfile.CUSTOM_TABLE_NAMES_DUT {
      if dev_id.to_uppercase().starts_with(&custom.dev_prefix) {
        table_name = custom.table_name.to_string();
        break;
      }
    }
  
    if table_name.len() == 0 {
      println!("Unknown DUT generation: {}", dev_id);
      return Ok("{}".to_string());
    }
  
    let mut dut_l1_calc = create_l1_calculator(&dev);
  
    let querier = QuerierDevIdTimestamp::new_diel_dev(table_name, dev_id.clone());
    let mut found_invalid_payload = false;
    let result = querier.run(&ts_ini, &ts_end, &mut |items| {
      for item in items {
        let payload = match get_raw_telemetry_pack_dut(&item) {
          Ok(v) => v,
          Err(err) => {
            if !found_invalid_payload {
              // println!("Ignoring invalid payload(s): {}\n{:?}", &err, item.to_string());
            }
            found_invalid_payload = true;
            continue;
          },
        };
        let result = split_pack(&payload, i_ts_ini, i_ts_end, dut_l1_calc.as_mut(), &mut |telemetry, index| {
          tcomp.AdcPontos(telemetry, index);
        }, &dev);
        match result {
          Ok(()) => {},
          Err(err) => {
            if !found_invalid_payload {
              // println!("Ignoring invalid payload(s): {}\n{:?}", &err, item.to_string());
            }
            found_invalid_payload = true;
            continue;
          },
        };
      }
      Ok(())
    }
  ).await;
  
    let mut provision_error = false;
    if let Err(err) = result {
      if err.starts_with("ProvisionedThroughputExceeded:") {
        provision_error = true;
      } else if err.starts_with("ResourceNotFound:") {
        // println!("Table not found for: {}", dev_id);
        return Ok("{}".to_string());
      } else {
        return Ok(format!("ERROR[117] {}", err).to_string());
      }
    }
  
    let mut accs = DutData {
      rpars: Some(serde_json::from_str(&rpars_serialized).unwrap()),
      page_ts_ini,
      tcomp,
    };
    
    let period_data = match accs.tcomp.CheckClosePeriod(isize::try_from(interval_length_s).unwrap(), check_minutes_offline, &ts_ini) {
        Err(err) => {println!("{}", err); return Ok("ERROR[120] CheckClosePeriod".to_string()); }
        Ok(v) => match v {
            Some(v) => v,
            None => {
                return Ok("{}".to_string());
            },
        }
    };
  
    let data = serde_json::json!({
      "hours_on": period_data.hours_online,
      "hours_on_l1": period_data.hours_on_l1,
      "lcmp": period_data.vec_l1,
    });
  
    Ok(data.to_string())
  }
  

pub fn parse_parameters_dut (dev_id: &str, temperature_offset: Option<f64>, day: &str, check_minutes_offline: Option<i32>) -> Result<ReqParameters, Box<dyn Error>> {
    if dev_id.len() < 9 {
      return Err(format!("ERROR[169] dev_id.len() < 9").into());
    }
  
    let interval_length_s = 24 * 60 * 60;
  
    let mut ts_ini = day;
  
    let i_ts_ini = match NaiveDateTime::parse_from_str(&format!("{}T00:00:00", ts_ini), "%Y-%m-%dT%H:%M:%S") {
      Err(err) => { println!("{} {}", &format!("{}T00:00:00", ts_ini), err); return Err(format!("Error parsing Date").into()) },
      Ok (mut date) => {
        date.timestamp()
      },
    };
  
    let ts_ini_aux = NaiveDateTime::from_timestamp(i_ts_ini, 0).format("%Y-%m-%dT%H:%M:%S").to_string();
    
    ts_ini = &ts_ini_aux;
  
    let i_ts_end = i_ts_ini + interval_length_s;

    let ts_end = NaiveDateTime::from_timestamp(i_ts_end + 120, 0).format("%Y-%m-%dT%H:%M:%S").to_string();
  
  
    let offset_temp = temperature_offset.unwrap_or(0.0);
  
    Ok(ReqParameters {
      dev_id: dev_id.to_string(),
      interval_length_s,
      ts_ini: ts_ini.to_string(),
      i_ts_ini,
      i_ts_end,
      ts_end,
      offset_temp,
      check_minutes_offline
    })
  }
  
struct DutData {
    pub rpars: Option<ReqParameters>,
    pub page_ts_ini: String,
    pub tcomp: DUTTelemetryCompiler,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DutHist {
  pub hours_on_l1: Decimal,
  pub hours_on: Decimal,
  pub lcmp: String,
}
