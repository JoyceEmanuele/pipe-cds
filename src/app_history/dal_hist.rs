use std::{error::Error, sync::Arc};

use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{compression::compiler_DAL::DALTelemetryCompiler, db::config::dynamo::QuerierDevIdTimestamp, telemetry_payloads::{dal_payload_json::get_raw_telemetry_pack_dal, dal_telemetry::split_pack}, GlobalVars};

pub fn parse_parameters_dal(day: &str, dev_id: &str, check_minutes_offline: Option<i32>) -> Result<ReqParameters, Box<dyn Error>> {
    if dev_id.len() < 9 {
        return Err("dev_id.len() < 9".into());
    }

    let interval_length_s = 24 * 60 * 60;
    let mut ts_ini = format!("{}{}", day, "T00:00:00");

    let i_ts_ini = match NaiveDateTime::parse_from_str(&ts_ini, "%Y-%m-%dT%H:%M:%S") {
        Err(err) => {
            println!("{} {}", &format!("{}T00:00:00", ts_ini), err);
            return Err("Error parsing Date".into());
        }
        Ok(date) => date.timestamp(),
    };

    ts_ini = NaiveDateTime::from_timestamp(i_ts_ini, 0)
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string();

    let i_ts_end = i_ts_ini + interval_length_s;
    let ts_end = NaiveDateTime::from_timestamp(i_ts_end, 0)
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string();

    Ok(ReqParameters {
        dev_id: dev_id.to_string(),
        interval_length_s,
        ts_ini: ts_ini.to_string(),
        i_ts_ini,
        i_ts_end,
        ts_end,
        check_minutes_offline,
    })
}

pub async fn process_comp_command_dal(
    rpars: ReqParameters,
    globs: &Arc<GlobalVars>,
) -> Result<String, Box<dyn Error>> {
    let rpars_serialized = serde_json::to_string(&rpars).unwrap();
    let dev_id = rpars.dev_id;
    let interval_length_s = rpars.interval_length_s;
    let ts_ini = rpars.ts_ini;
    let i_ts_ini = rpars.i_ts_ini;
    let i_ts_end = rpars.i_ts_end;
    let ts_end = rpars.ts_end;
    let check_minutes_offline = rpars.check_minutes_offline;

    let accs: DalData = {
        let page_ts_ini = ts_ini.clone();
        let tcomp = DALTelemetryCompiler::new(interval_length_s);
        DalData {
            rpars: None,
            page_ts_ini,
            tcomp,
        }
    };

    let page_ts_ini = accs.page_ts_ini;
    let mut tcomp = accs.tcomp;

    let mut table_name = {
        if (dev_id.len() == 12) && dev_id.to_uppercase().starts_with("DAL") {
            format!("{}XXXX_RAW", &dev_id.to_uppercase()[0..8])
        } else {
            String::new()
        }
    };

    for custom in &globs.configfile.CUSTOM_TABLE_NAMES_DAL {
        if dev_id.to_uppercase().starts_with(&custom.dev_prefix) {
            table_name = custom.table_name.to_owned();
            break;
        }
    }

    if table_name.len() == 0 {
        println!("Unknown DAL generation: {}", dev_id);
        return Ok("{}".to_string());
    }

    let querier = QuerierDevIdTimestamp::new_diel_dev(
        table_name,
        dev_id.clone(),
    );

    let mut found_invalid_payload = false;
    let result = querier
        .run(&ts_ini, &ts_end, &mut |items| {
            for item in items {
                let payload = match get_raw_telemetry_pack_dal(&item) {
                    Ok(v) => v,
                    Err(err) => {
                        if !found_invalid_payload {
                            // println!("Ignoring invalid payload(s): {}\n{:?}", &err, item.to_string());
                        }
                        found_invalid_payload = true;
                        continue;
                    }
                };
                let result = split_pack(&payload, i_ts_ini, i_ts_end, &mut |telemetry, index| {
                    tcomp.AdcPontos(telemetry, index);
                });
                match result {
                    Ok(()) => {}
                    Err(err) => {
                        if !found_invalid_payload {
                            // println!("Ignoring invalid payload(s): {}\n{:?}", &err, item.to_string());
                        }
                        found_invalid_payload = true;
                        continue;
                    }
                };
            }
            return Ok(());
        })
        .await;

    let mut provision_error = false;
    if let Err(err) = result {
        if err.starts_with("ProvisionedThroughputExceeded:") {
            provision_error = true;
        } else if err.starts_with("ResourceNotFound:") {
            // println!("Table not found for: {}", dev_id);
            return Ok("{}".to_string());
        } else {
            return Ok(format!("ERROR[78] {}", err).to_string());
        }
    }

    let mut accs = DalData {
        rpars: Some(serde_json::from_str(&rpars_serialized).unwrap()),
        page_ts_ini,
        tcomp,
    };

    let period_data = match accs.tcomp.CheckClosePeriod(isize::try_from(interval_length_s).unwrap(), check_minutes_offline, &ts_ini) {
        Err(err) => { println!("{}", err); return Ok("ERROR[120] CheckClosePeriod".to_string()); },
        Ok(v) => match v {
            Some(v) => v,
            None => {
              return Ok("{}".to_string());
            },
          }
    };

    let mut data = serde_json::json!({});

    data["hours_online"] = period_data.hours_online.into();

    Ok(data.to_string())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReqParameters {
    pub dev_id: String,
    pub interval_length_s: i64,
    pub ts_ini: String,
    pub i_ts_ini: i64,
    pub i_ts_end: i64,
    pub ts_end: String,
    pub check_minutes_offline: Option<i32>,
}

#[derive(Serialize, Deserialize)]
struct DalData {
  pub rpars: Option<ReqParameters>,
  pub page_ts_ini: String,
  pub tcomp: DALTelemetryCompiler,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DalHist {
    pub hours_online: Decimal,
}
