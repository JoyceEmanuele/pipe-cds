use chrono::{Duration, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::{error::Error, sync::Arc};

use crate::{
    compression::compiler_DAC::DACTelemetryCompiler, db::config::dynamo::QuerierDevIdTimestamp, models::external_models::device::DacDevice, telemetry_payloads::{dac_payload_json::get_raw_telemetry_pack_dac, dac_telemetry::{split_pack, HwInfoDAC, T_sensor_cfg, T_sensors}}, GlobalVars
};

pub fn parse_parameters_dac(
    parsed: &DacDevice,
    day: &str,
    check_minutes_offline: Option<i32>
) -> Result<ReqParameters, Box<dyn Error>> {
    let dev_id = parsed.device_code.to_owned();

    if dev_id.len() < 9 {
        return Err("dev_id.len() < 9".into());
    }

    let interval_length_s = 24 * 60 * 60;

    let mut ts_ini = format!("{}{}", day, "T00:00:00");

    let i_ts_ini = match NaiveDateTime::parse_from_str(&ts_ini, "%Y-%m-%dT%H:%M:%S") {
        Err(err) => {
            println!("{} {}", ts_ini, err);
            return Err(format!("Error parsing Date").into());
        }
        Ok(mut date) => {
            date.timestamp()
        }
    };

    let ts_ini_aux = NaiveDateTime::from_timestamp(i_ts_ini, 0)
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string();

    ts_ini = ts_ini_aux;

    let i_ts_end = i_ts_ini + interval_length_s;

    let ts_end = NaiveDateTime::from_timestamp(i_ts_end + 60, 0)
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string();

    let hw_cfg = HwInfoDAC {
        isVrf: parsed.is_vrf,
        calculate_L1_fancoil: Some(parsed.calculate_l1_fancoil),
        hasAutomation: parsed.has_automation,
        P0Psuc: parsed.p0_psuc,
        P1Psuc: parsed.p1_psuc,
        P0Pliq: parsed.p0_pliq,
        P1Pliq: parsed.p1_pliq,
        P0mult: parsed.p0_mult.unwrap_or(0.0),
        P0ofst: parsed.p0_ofst.unwrap_or(0.0),
        P1mult: parsed.p1_mult.unwrap_or(0.0),
        P1ofst: parsed.p1_ofst.unwrap_or(0.0),
        fluid: parsed.fluid_type.clone(),
        t_cfg: match &parsed.t0_t1_t2 {
            None => None,
            Some(T0_T1_T2) => {
            if T0_T1_T2.len() != 3 { return Err("Invalid T0_T1_T2".into()); }
              let Tamb = if let "Tamb" = T0_T1_T2[0].as_str() { Some(T_sensors::T0) } else if let "Tamb" = T0_T1_T2[1].as_str() { Some(T_sensors::T1) } else if let "Tamb" = T0_T1_T2[2].as_str() { Some(T_sensors::T2) } else { None };
              let Tsuc = if let "Tsuc" = T0_T1_T2[0].as_str() { Some(T_sensors::T0) } else if let "Tsuc" = T0_T1_T2[1].as_str() { Some(T_sensors::T1) } else if let "Tsuc" = T0_T1_T2[2].as_str() { Some(T_sensors::T2) } else { None };
              let Tliq = if let "Tliq" = T0_T1_T2[0].as_str() { Some(T_sensors::T0) } else if let "Tliq" = T0_T1_T2[1].as_str() { Some(T_sensors::T1) } else if let "Tliq" = T0_T1_T2[2].as_str() { Some(T_sensors::T2) } else { None };
              Some(T_sensor_cfg{
                Tamb,
                Tsuc,
                Tliq,
              })
            }
        },
        simulate_l1: parsed.virtual_l1,
    };

    if hw_cfg.P0Psuc || hw_cfg.P0Pliq {
        if parsed.p0_mult.is_none() {
            return Err("Missing P0mult".into());
        }
        if parsed.p0_ofst.is_none() {
            return Err("Missing P0ofst".into());
        }
    }
    if hw_cfg.P1Psuc || hw_cfg.P1Pliq {
        if parsed.p1_mult.is_none() {
            return Err("Missing P1mult".into());
        }
        if parsed.p1_ofst.is_none() {
            return Err("Missing P1ofst".into());
        }
    }

    let open_end = true;

    return Ok(ReqParameters {
        hw_cfg,
        dev_id: dev_id.to_string(),
        interval_length_s,
        ts_ini: ts_ini.to_string(),
        i_ts_ini,
        i_ts_end,
        ts_end,
        open_end,
        check_minutes_offline,
    });
}

pub async fn process_comp_command_dac_v2(
    rpars: ReqParameters,
    globs: &Arc<GlobalVars>,
) -> Result<String, Box<dyn Error>> {
    let rpars_serialized = serde_json::to_string(&rpars).unwrap();
    let hw_cfg = rpars.hw_cfg;
    let dev_id = rpars.dev_id;
    let interval_length_s = rpars.interval_length_s;
    let ts_ini = rpars.ts_ini;

    // Processa 15 minutos antes do período para preparar o estado do L1.
    // Não atualizamos i_ts_ini pois é usado para identificar os limites do gráfico
    // e não queremos que esses 15min vão para o gráfico.
    let ts_ini_aux = NaiveDateTime::parse_from_str(&ts_ini, "%Y-%m-%dT%H:%M:%S").map_err(|e| e.to_string())?;

    let ts_ini = {
        let ts = NaiveDateTime::parse_from_str(&ts_ini, "%Y-%m-%dT%H:%M:%S")
            .map_err(|e| e.to_string())?;

        let ts = ts - chrono::Duration::minutes(15);
        ts.to_string()
    };

    let i_ts_ini = rpars.i_ts_ini;
    let i_ts_end = rpars.i_ts_end;
    let ts_end = rpars.ts_end;
    let open_end = rpars.open_end;
    let check_minutes_offline = rpars.check_minutes_offline;

    let accs: DacData = {
        let page_ts_ini = ts_ini.clone();
        let tcomp = DACTelemetryCompiler::new(rpars.interval_length_s, &hw_cfg);
        DacData {
            rpars: None,
            page_ts_ini,
            tcomp,
        }
    };

    let mut page_ts_ini = accs.page_ts_ini;
    let mut tcomp = accs.tcomp;

    let mut table_name = {
        if (dev_id.len() == 12) && dev_id.to_uppercase().starts_with("DAC") {
            format!("{}XXXX_RAW", &dev_id.to_uppercase()[0..8])
        } else {
            String::new()
        }
    };

    for custom in &globs.configfile.CUSTOM_TABLE_NAMES_DAC {
        if dev_id.to_uppercase().starts_with(&custom.dev_prefix) {
            table_name = custom.table_name.to_owned();
            break;
        }
    }

    if table_name.len() == 0 {
        println!("Unknown DAC generation: {}", dev_id);
        return Ok("{}".to_string());
    }
    let mut dac_state =
        crate::telemetry_payloads::dac_l1::dac_l1_calculator::create_l1_calculator(&hw_cfg);

    let querier = if table_name == "DAC20719XXXX_RAW" {
        QuerierDevIdTimestamp::new_custom(
            table_name,
            "dac_id".to_owned(),
            "timestamp".to_owned(),
            dev_id.clone(),
        )
    } else {
        QuerierDevIdTimestamp::new_diel_dev(
            table_name,
            dev_id.clone(),
        )
    };
    let mut found_invalid_payload = false;
    let result = querier
        .run(&ts_ini, &ts_end, &mut |items| {
            for item in items {
                let payload = match get_raw_telemetry_pack_dac(&item) {
                    Ok(v) => v,
                    Err(err) => {
                        if !found_invalid_payload {
                            // println!(
                            //     "Ignoring invalid payload(s): {}\n{:?}",
                            //     &err,
                            //     item.to_string()
                            // );
                        }
                        found_invalid_payload = true;
                        continue;
                    }
                };
                let result = split_pack(
                    &payload,
                    i_ts_ini,
                    i_ts_end,
                    &hw_cfg,
                    dac_state.as_mut(),
                    &mut |telemetry, index| {
                        tcomp.AdcPontos(telemetry, index, payload.samplingTime);
                    },
                );
                match result {
                    Ok(()) => {}
                    Err(err) => {
                        if !found_invalid_payload {
                            // println!(
                            //     "Ignoring invalid payload(s): {}\n{:?}",
                            //     &err,
                            //     item.to_string()
                            // );
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
            return Ok(format!("ERROR[117] {}", err).to_string());
        }
    }

    let mut accs = DacData {
        rpars: Some(serde_json::from_str(&rpars_serialized).unwrap()),
        page_ts_ini,
        tcomp,
    };

    let period_data = match accs.tcomp.CheckClosePeriod(if open_end {
        accs.tcomp.last_index + 1
    } else {
        isize::try_from(interval_length_s).unwrap()
    }, check_minutes_offline, &ts_ini_aux.to_string()) {
        Err(err) => {
            println!("{}", err);
            return Ok("ERROR[120] CheckClosePeriod".to_string());
        }
        Ok(v) => match v {
            Some(v) => v,
            None => {
                return Ok("{}".to_string());
            }
        },
    };

    let mut data = serde_json::json!({});

    data["hours_on"] = period_data.hours_on.into();
    data["hours_dev_on"] = period_data.hours_dev_on.into();
    data["lcmp"] = period_data.vec_l1.into();
    data["last_telemetry_time"] = period_data.last_telemetry_time.into();

    Ok(data.to_string())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReqParameters {
    pub hw_cfg: HwInfoDAC,
    pub dev_id: String,
    pub interval_length_s: i64,
    pub ts_ini: String,
    pub i_ts_ini: i64,
    pub i_ts_end: i64,
    pub ts_end: String,
    pub open_end: bool,
    pub check_minutes_offline: Option<i32>,
}

#[derive(Serialize, Deserialize)]
struct DacData {
    pub rpars: Option<ReqParameters>,
    pub page_ts_ini: String,
    pub tcomp: DACTelemetryCompiler,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DacHist {
  pub hours_on: Decimal,
  pub lcmp: String,
  pub hours_dev_on: Decimal,
  pub last_telemetry_time: String,
}
