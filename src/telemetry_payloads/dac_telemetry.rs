use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use super::{dac_l1::dac_l1_calculator::DacL1Calculator, telemetry_formats::{TelemetryDAC_v3, TelemetryDACv2, TelemetryPackDAC_v2}};


#[derive(Serialize, Deserialize, Debug)]
pub enum T_sensors { T0, T1, T2 }

#[derive(Serialize, Deserialize, Debug)]
pub struct T_sensor_cfg {
  pub Tamb: Option<T_sensors>,
  pub Tsuc: Option<T_sensors>,
  pub Tliq: Option<T_sensors>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HwInfoDAC {
  pub isVrf: bool,
  pub calculate_L1_fancoil: Option<bool>,
  pub hasAutomation: bool,
  pub P0Psuc: bool,
  pub P1Psuc: bool,
  pub P0Pliq: bool,
  pub P1Pliq: bool,
  pub P0mult: f64,
  pub P0ofst: f64,
  pub P1mult: f64,
  pub P1ofst: f64,
  pub fluid: Option<String>,
  pub t_cfg: Option<T_sensor_cfg>,
  #[serde(rename="simulateL1")]
  pub simulate_l1: bool,
}

pub fn split_pack(payload: &TelemetryPackDAC_v2, ts_ini: i64, ts_next: i64, dev: &HwInfoDAC, dac_state: &mut dyn DacL1Calculator, itemCallback: &mut dyn FnMut(&mut TelemetryDAC_v3, isize)) -> Result<(),String> {
    if payload.T0.len() != payload.L1.len() { return Err(format!("Incompatible length of T0 at {}", payload.timestamp)) }
    if payload.T1.len() != payload.L1.len() { return Err(format!("Incompatible length of T1 at {}", payload.timestamp)) }
    if payload.T2.len() != payload.L1.len() { return Err(format!("Incompatible length of T2 at {}", payload.timestamp)) }
    if payload.P0.len() != payload.L1.len() { return Err(format!("Incompatible length of P0 at {}", payload.timestamp)) }
    if payload.P1.len() != payload.L1.len() { return Err(format!("Incompatible length of P1 at {}", payload.timestamp)) }
  
    let pack_ts = match NaiveDateTime::parse_from_str(&payload.timestamp, "%Y-%m-%dT%H:%M:%S") {
      Err(_) => {
        println!("Error parsing Date:\n{:?}", payload);
        return Err("Error parsing Date".to_owned());
      },
      Ok (date) => date.timestamp(),
    };
    let sampling_time: i64 = payload.samplingTime; // de quantos em quantos segundos o firmware lê os sensores e insere nos vetores.
  
    let mut telemetry = TelemetryDAC_v3{
      timestamp: payload.timestamp.clone(),
      Lcmp: None,
      Tamb: None,
      Tsuc: None,
      Tliq: None,
      Psuc: None,
      Pliq: None,
      saved_data: payload.saved_data,
    };
    
    let mut remainingSteps = payload.L1.len();
    for _i in 0..payload.L1.len() {
      let telm_ts = pack_ts - ((remainingSteps as i64 - 1) * sampling_time);
      remainingSteps = checkSetTelemetryValues(dev, payload, &mut telemetry, telm_ts, dac_state, remainingSteps);
      let index = payload.L1.len() - 1 - remainingSteps;
      if telm_ts < ts_ini { continue; }
      if telm_ts >= ts_next { continue; }
      itemCallback(&mut telemetry, isize::try_from(telm_ts - ts_ini).unwrap());
    }
  
    return Ok(());
}

pub fn checkSetTelemetryValues(dev: &HwInfoDAC, payload: &TelemetryPackDAC_v2, telemetry: &mut TelemetryDAC_v3, current_ts: i64, dac_state: &mut dyn DacL1Calculator, mut remainingSteps: usize) -> usize {
  if remainingSteps == 0 {
    remainingSteps = payload.L1.len() - 1;
  } else { remainingSteps -= 1; }
  let index = payload.L1.len() - 1 - remainingSteps;

  if let Some(t_cfg) = &dev.t_cfg {
    let T0 = match payload.T0[index] { None => None, Some(T0) => { if (T0 <= -99.0) || (T0 >= 85.0) { None } else { Some(T0) } } };
    let T1 = match payload.T1[index] { None => None, Some(T1) => { if (T1 <= -99.0) || (T1 >= 85.0) { None } else { Some(T1) } } };
    let T2 = match payload.T2[index] { None => None, Some(T2) => { if (T2 <= -99.0) || (T2 >= 85.0) { None } else { Some(T2) } } };
    telemetry.Tamb = match t_cfg.Tamb { None => None, Some(T_sensors::T0) => T0, Some(T_sensors::T1) => T1, Some(T_sensors::T2) => T2, };
    telemetry.Tsuc = match t_cfg.Tsuc { None => None, Some(T_sensors::T0) => T0, Some(T_sensors::T1) => T1, Some(T_sensors::T2) => T2, };
    telemetry.Tliq = match t_cfg.Tliq { None => None, Some(T_sensors::T0) => T0, Some(T_sensors::T1) => T1, Some(T_sensors::T2) => T2, };
  } else {
    telemetry.Tsuc = match payload.T1[index] {
      None => None,
      Some(T1) => {
        if T1 <= -99.0 || T1 >= 85.0 { None }
        else { Some(T1) }
      }
    };
    telemetry.Tliq = match payload.T2[index] {
      None => None,
      Some(T2) => {
        if T2 <= -99.0 || T2 >= 85.0 { None }
        else { Some(T2) }
      }
    };
    telemetry.Tamb = match payload.T0[index] {
      None => None,
      Some(T0) => {
        if T0 <= -99.0 || T0 >= 85.0 { None }
        else { Some(T0) }
      }
    };
  }

  telemetry.Psuc = {
    if dev.P0Psuc      { payload.P0[index].map(|P0| ((f64::from(P0) * dev.P0mult + dev.P0ofst) * 10.).round() / 10.) }
    else if dev.P1Psuc { payload.P1[index].map(|P1| ((f64::from(P1) * dev.P1mult + dev.P1ofst) * 10.).round() / 10.) }
    else                 { None }
  };
  telemetry.Pliq = {
    if dev.P0Pliq      { payload.P0[index].map(|P0| ((f64::from(P0) * dev.P0mult + dev.P0ofst) * 10.).round() / 10.) }
    else if dev.P1Pliq { payload.P1[index].map(|P1| ((f64::from(P1) * dev.P1mult + dev.P1ofst) * 10.).round() / 10.) }
    else                 { None }
  };

  let single_point_payload = TelemetryDACv2 {
    l1: payload.L1[index],
    timestamp: NaiveDateTime::from_timestamp_opt(current_ts, 0).unwrap(),
    p0: payload.P0[index],
    p1: payload.P1[index],
  };

  let l1 = dac_state
    .calc_l1(telemetry, &single_point_payload, dev)
    .ok()
    .flatten();

  if dev.hasAutomation {
    match payload.State.as_deref() {
      Some("Disabled") => {
        telemetry.Lcmp = Some(false);
      },
      Some("Enabled") => {
        telemetry.Lcmp = l1;
      },
      _ => {
        telemetry.Lcmp = None;
      }
    }
  }
  else {
    telemetry.Lcmp = l1;
  }

  return remainingSteps;
}
