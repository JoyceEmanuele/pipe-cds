use std::{borrow::Cow, collections::HashMap};

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::telemetry_payloads::energy::padronized::calculate_formulas;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TelemetryDri<'a> {
    pub dev_id: Cow<'a, String>,
    pub timestamp: Cow<'a, String>,
    #[serde(rename="type")]
    pub dev_type: Cow<'a, String>,
    pub values: Option<Vec<Option<i16>>>,
    #[serde(rename="therm-on")]
    pub therm_on: Option<i16>,
    pub fanspeed: Option<i16>,
    pub mode: Option<i16>,
    pub setpoint: Option<i16>,
    pub lock: Option<i16>,
    #[serde(rename="temp-amb")]
    pub temp_amb: Option<i16>,
    #[serde(rename="valve-on")]
    pub valve_on: Option<i16>,
    #[serde(rename="fan-status")]
    pub fan_status: Option<i16>,
    pub formulas: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DriCCNTelemetry {
    pub timestamp: String,
    pub Setpoint: Option<i16>,
    pub Status: Option<i16>,
    pub Mode: Option<i16>,
}

#[derive(Debug)]
pub struct HwInfoDRI {
    pub formulas: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DriVAVandFancoilTelemetry {
    pub timestamp: String,
    pub ThermOn: Option<f64>,
    pub Fanspeed: Option<f64>,
    pub Mode: Option<f64>,
    pub Setpoint: Option<f64>,
    pub Lock: Option<f64>,
    pub TempAmb: Option<f64>,
    pub ValveOn: Option<f64>,
    pub FanStatus: Option<f64>,
}

impl<'a> TryFrom<TelemetryDri<'a>> for DriCCNTelemetry {
    type Error = String;
    fn try_from(value: TelemetryDri) -> Result<DriCCNTelemetry, String> {
        if value.dev_type.to_string() != String::from("CCN") {
            return Err("The dev type and telemetry type does not match".to_string())
        }
        if value.values.is_none() {
            return Err("Telemetry does not have \"values\" field".to_string())
        }
        let values = value.values.unwrap();
        let result = DriCCNTelemetry {
            timestamp: value.timestamp.to_string(),
            Setpoint: match values[0] {
                Some(-1) => None,
                _ => values[0],
            },
            Status: match values[1] {
                Some(-1) => None,
                _ => values[1],
            },
            Mode: match values[2] {
                Some(-1) => None,
                _ => values[2],
            },
        };
        Ok(result)
    }
}

impl<'a> TryFrom<TelemetryDri<'a>> for DriVAVandFancoilTelemetry {
    type Error = String;
    fn try_from(value: TelemetryDri) -> Result<DriVAVandFancoilTelemetry, String> {
        // if !value.dev_type.to_string().starts_with("VAV") {
        //     return Err("The dev type and telemetry type does not match".to_string())
        // }
        let tel = json!(value);

        let result = DriVAVandFancoilTelemetry {
            timestamp: value.timestamp.to_string(),
            ThermOn: match value.therm_on {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("therm-on", value.therm_on.unwrap() as f64, &tel, false)),
            },
            Fanspeed: match value.fanspeed {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("fanspeed", value.fanspeed.unwrap() as f64, &tel, false)),
            },
            Mode: match value.mode {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("mode", value.mode.unwrap() as f64, &tel, false)),
            },
            Setpoint: match value.setpoint {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("setpoint", value.setpoint.unwrap() as f64, &tel, false)),
            },
            Lock: match value.lock {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("lock", value.lock.unwrap() as f64, &tel, false)),
            },
            TempAmb: match value.temp_amb {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("temp-amb", value.temp_amb.unwrap() as f64, &tel, false)),
            },
            ValveOn: match value.valve_on {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("valve-on", value.valve_on.unwrap() as f64, &tel, false)),
            },
            FanStatus: match value.fan_status {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("fan-status", value.fan_status.unwrap() as f64, &tel, false)),
            },
        };
        Ok(result)
    }
}

pub fn split_pack_ccn (mut payload: &DriCCNTelemetry, ts_ini: i64, ts_next: i64, itemCallback: &mut dyn FnMut(&DriCCNTelemetry, isize)) -> Result<(),String> {
    let pack_ts = match NaiveDateTime::parse_from_str(&payload.timestamp, "%Y-%m-%dT%H:%M:%S") {
      Err(_) => {
        println!("Error parsing Date:\n{:?}", payload);
        return Err("Error parsing Date".to_owned());
      },
      Ok (date) => date.timestamp(),
    };
  
    if (pack_ts < ts_ini) || (pack_ts >= ts_next) { } // ignore
    else {
      itemCallback(&mut payload, isize::try_from(pack_ts - ts_ini).unwrap());
    }
  
    return Ok(());
}

pub fn split_pack_vav_and_fancoil (mut payload: &DriVAVandFancoilTelemetry, ts_ini: i64, ts_next: i64, itemCallback: &mut dyn FnMut(&DriVAVandFancoilTelemetry, isize)) -> Result<(),String> {
    let pack_ts = match NaiveDateTime::parse_from_str(&payload.timestamp, "%Y-%m-%dT%H:%M:%S") {
      Err(_) => {
        println!("Error parsing Date:\n{:?}", payload);
        return Err("Error parsing Date".to_owned());
      },
      Ok (date) => date.timestamp(),
    };
  
    if (pack_ts < ts_ini) || (pack_ts >= ts_next) { } // ignore
    else {
      itemCallback(&mut payload, isize::try_from(pack_ts - ts_ini).unwrap());
    }
  
    return Ok(());
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TelemetryDriChillerCarrierHX<'a> {
    pub dev_id: Cow<'a, String>,
    pub timestamp: Cow<'a, String>,
    #[serde(rename="type")]
    pub dev_type: Cow<'a, String>,
    pub CHIL_S_S: Option<i16>,
    pub ALM: Option<i16>,
    pub alarm_1: Option<i16>,
    pub alarm_2: Option<i16>,
    pub alarm_3: Option<i16>,
    pub alarm_4: Option<i16>,
    pub alarm_5: Option<i16>,
    pub CAP_T: Option<i16>,
    pub DEM_LIM: Option<i16>,
    pub LAG_LIM: Option<i16>,
    pub SP: Option<i16>,
    pub CTRL_PNT: Option<i16>,
    pub EMSTOP: Option<i16>,
    pub CP_A1: Option<i16>,
    pub CP_A2: Option<i16>,
    pub CAPA_T: Option<i16>,
    pub DP_A: Option<i16>,
    pub SP_A: Option<i16>,
    pub SCT_A: Option<i16>,
    pub SST_A: Option<i16>,
    pub CP_B1: Option<i16>,
    pub CP_B2: Option<i16>,
    pub CAPB_T: Option<i16>,
    pub DP_B: Option<i16>,
    pub SP_B: Option<i16>,
    pub SCT_B: Option<i16>,
    pub SST_B: Option<i16>,
    pub COND_LWT: Option<i16>,
    pub COND_EWT: Option<i16>,
    pub COOL_LWT: Option<i16>,
    pub COOL_EWT: Option<i16>,
    pub CPA1_OP: Option<i16>,
    pub CPA2_OP: Option<i16>,
    pub DOP_A1: Option<i16>,
    pub DOP_A2: Option<i16>,
    pub CPA1_DGT: Option<i16>,
    pub CPA2_DGT: Option<i16>,
    pub EXV_A: Option<i16>,
    pub HR_CP_A1: Option<i16>,
    pub HR_CP_A2: Option<i16>,
    pub CPA1_TMP: Option<i16>,
    pub CPA2_TMP: Option<i16>,
    pub CPA1_CUR: Option<i16>,
    pub CPA2_CUR: Option<i16>,
    pub CPB1_OP: Option<i16>,
    pub CPB2_OP: Option<i16>,
    pub DOP_B1: Option<i16>,
    pub DOP_B2: Option<i16>,
    pub CPB1_DGT: Option<i16>,
    pub CPB2_DGT: Option<i16>,
    pub EXV_B: Option<i16>,
    pub HR_CP_B1: Option<i16>,
    pub HR_CP_B2: Option<i16>,
    pub CPB1_TMP: Option<i16>,
    pub CPB2_TMP: Option<i16>,
    pub CPB1_CUR: Option<i16>,
    pub CPB2_CUR: Option<i16>,
    pub COND_SP: Option<i16>,
    pub CHIL_OCC: Option<i16>,
    pub STATUS: Option<i16>,
    pub formulas: Option<HashMap<String, String>>,
    pub setpoint: Option<i16>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DriChillerCarrierHXTelemetry {
    pub timestamp: NaiveDateTime,
    pub CHIL_S_S: Option<f64>,
    pub ALM: Option<f64>,
    pub alarm_1: Option<f64>,
    pub alarm_2: Option<f64>,
    pub alarm_3: Option<f64>,
    pub alarm_4: Option<f64>,
    pub alarm_5: Option<f64>,
    pub CAP_T: Option<f64>,
    pub DEM_LIM: Option<f64>,
    pub LAG_LIM: Option<f64>,
    pub SP: Option<f64>,
    pub CTRL_PNT: Option<f64>,
    pub EMSTOP: Option<f64>,
    pub CP_A1: Option<f64>,
    pub CP_A2: Option<f64>,
    pub CAPA_T: Option<f64>,
    pub DP_A: Option<f64>,
    pub SP_A: Option<f64>,
    pub SCT_A: Option<f64>,
    pub SST_A: Option<f64>,
    pub CP_B1: Option<f64>,
    pub CP_B2: Option<f64>,
    pub CAPB_T: Option<f64>,
    pub DP_B: Option<f64>,
    pub SP_B: Option<f64>,
    pub SCT_B: Option<f64>,
    pub SST_B: Option<f64>,
    pub COND_LWT: Option<f64>,
    pub COND_EWT: Option<f64>,
    pub COOL_LWT: Option<f64>,
    pub COOL_EWT: Option<f64>,
    pub CPA1_OP: Option<f64>,
    pub CPA2_OP: Option<f64>,
    pub DOP_A1: Option<f64>,
    pub DOP_A2: Option<f64>,
    pub CPA1_DGT: Option<f64>,
    pub CPA2_DGT: Option<f64>,
    pub EXV_A: Option<f64>,
    pub HR_CP_A1: Option<f64>,
    pub HR_CP_A2: Option<f64>,
    pub CPA1_TMP: Option<f64>,
    pub CPA2_TMP: Option<f64>,
    pub CPA1_CUR: Option<f64>,
    pub CPA2_CUR: Option<f64>,
    pub CPB1_OP: Option<f64>,
    pub CPB2_OP: Option<f64>,
    pub DOP_B1: Option<f64>,
    pub DOP_B2: Option<f64>,
    pub CPB1_DGT: Option<f64>,
    pub CPB2_DGT: Option<f64>,
    pub EXV_B: Option<f64>,
    pub HR_CP_B1: Option<f64>,
    pub HR_CP_B2: Option<f64>,
    pub CPB1_TMP: Option<f64>,
    pub CPB2_TMP: Option<f64>,
    pub CPB1_CUR: Option<f64>,
    pub CPB2_CUR: Option<f64>,
    pub COND_SP: Option<f64>,
    pub CHIL_OCC: Option<f64>,
    pub STATUS: Option<f64>,
}

impl DriChillerCarrierHXTelemetry {
    pub fn set_field_average(&mut self, field: &str, value: f64) {
        match field {
            "CAP_T" => self.CAP_T = Some(value),
            "DEM_LIM" => self.DEM_LIM = Some(value),
            "LAG_LIM" => self.LAG_LIM = Some(value),
            "SP" => self.SP = Some(value),
            "CTRL_PNT" => self.CTRL_PNT = Some(value),
            "CAPA_T" => self.CAPA_T = Some(value),
            "DP_A" => self.DP_A = Some(value),
            "SP_A" => self.SP_A = Some(value),
            "SCT_A" => self.SCT_A = Some(value),
            "SST_A" => self.SST_A = Some(value),
            "CAPB_T" => self.CAPB_T = Some(value),
            "DP_B" => self.DP_B = Some(value),
            "SP_B" => self.SP_B = Some(value),
            "SCT_B" => self.SCT_B = Some(value),
            "SST_B" => self.SST_B = Some(value),
            "COND_LWT" => self.COND_LWT = Some(value),
            "COND_EWT" => self.COND_EWT = Some(value),
            "COOL_LWT" => self.COOL_LWT = Some(value),
            "COOL_EWT" => self.COOL_EWT = Some(value),
            "CPA1_OP" => self.CPA1_OP = Some(value),
            "CPA2_OP" => self.CPA2_OP = Some(value),
            "DOP_A1" => self.DOP_A1 = Some(value),
            "DOP_A2" => self.DOP_A2 = Some(value),
            "CPA1_DGT" => self.CPA1_DGT = Some(value),
            "CPA2_DGT" => self.CPA2_DGT = Some(value),
            "EXV_A" => self.EXV_A = Some(value),
            "HR_CP_A1" => self.HR_CP_A1 = Some(value),
            "HR_CP_A2" => self.HR_CP_A2 = Some(value),
            "CPA1_TMP" => self.CPA1_TMP = Some(value),
            "CPA2_TMP" => self.CPA2_TMP = Some(value),
            "CPA1_CUR" => self.CPA1_CUR = Some(value),
            "CPA2_CUR" => self.CPA2_CUR = Some(value),
            "CPB1_OP" => self.CPB1_OP = Some(value),
            "CPB2_OP" => self.CPB2_OP = Some(value),
            "DOP_B1" => self.DOP_B1 = Some(value),
            "DOP_B2" => self.DOP_B2 = Some(value),
            "CPB1_DGT" => self.CPB1_DGT = Some(value),
            "CPB2_DGT" => self.CPB2_DGT = Some(value),
            "EXV_B" => self.EXV_B = Some(value),
            "HR_CP_B1" => self.HR_CP_B1 = Some(value),
            "HR_CP_B2" => self.HR_CP_B2 = Some(value),
            "CPB1_TMP" => self.CPB1_TMP = Some(value),
            "CPB2_TMP" => self.CPB2_TMP = Some(value),
            "CPB1_CUR" => self.CPB1_CUR = Some(value),
            "CPB2_CUR" => self.CPB2_CUR = Some(value),
            "COND_SP" => self.COND_SP = Some(value),
            _ => (),
        }
    }
    
    pub fn new(timestamp: NaiveDateTime) -> Self {
        Self {
            timestamp,
            CHIL_S_S: None,
            ALM: None,
            alarm_1: None,
            alarm_2: None,
            alarm_3: None,
            alarm_4: None,
            alarm_5: None,
            CAP_T: None,
            DEM_LIM: None,
            LAG_LIM: None,
            SP: None,
            CTRL_PNT: None,
            EMSTOP: None,
            CAPA_T: None,
            DP_A: None,
            SP_A: None,
            SCT_A: None,
            SST_A: None,
            CAPB_T: None,
            DP_B: None,
            SP_B: None,
            SCT_B: None,
            SST_B: None,
            COND_LWT: None,
            COND_EWT: None,
            COOL_LWT: None,
            COOL_EWT: None,
            CPA1_OP: None,
            CPA2_OP: None,
            DOP_A1: None,
            DOP_A2: None,
            CPA1_DGT: None,
            CPA2_DGT: None,
            EXV_A: None,
            HR_CP_A1: None,
            HR_CP_A2: None,
            CPA1_TMP: None,
            CPA2_TMP: None,
            CPA1_CUR: None,
            CPA2_CUR: None,
            CPB1_OP: None,
            CPB2_OP: None,
            DOP_B1: None,
            DOP_B2: None,
            CPB1_DGT: None,
            CPB2_DGT: None,
            EXV_B: None,
            HR_CP_B1: None,
            HR_CP_B2: None,
            CPB1_TMP: None,
            CPB2_TMP: None,
            CPB1_CUR: None,
            CPB2_CUR: None,
            COND_SP: None,
            CHIL_OCC: None,
            CP_A1: None,
            CP_A2: None,
            CP_B1: None,
            CP_B2: None,
            STATUS: None,
        }
    }
}

impl<'a> TryFrom<TelemetryDriChillerCarrierHX<'a>> for DriChillerCarrierHXTelemetry {
    type Error = String;
    fn try_from(value: TelemetryDriChillerCarrierHX) -> Result<DriChillerCarrierHXTelemetry, String> {
        let tel = json!(value);

        let result = DriChillerCarrierHXTelemetry {
            timestamp: NaiveDateTime::parse_from_str(value.timestamp.as_ref(), "%Y-%m-%dT%H:%M:%S")
            .map_err(|e| e.to_string())?,
            CHIL_S_S: match value.CHIL_S_S {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CHIL_S_S", value.CHIL_S_S.unwrap() as f64, &tel, false)),
            },
            ALM: match value.ALM {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("ALM", value.ALM.unwrap() as f64, &tel, false)),
            },
            alarm_1: match value.alarm_1 {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("alarm_1", value.alarm_1.unwrap() as f64, &tel, false)),
            },
            alarm_2: match value.alarm_2 {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("alarm_2", value.alarm_2.unwrap() as f64, &tel, false)),
            },
            alarm_3: match value.alarm_3 {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("alarm_3", value.alarm_3.unwrap() as f64, &tel, false)),
            },
            alarm_4: match value.alarm_4 {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("alarm_4", value.alarm_4.unwrap() as f64, &tel, false)),
            },
            alarm_5: match value.alarm_5 {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("alarm_5", value.alarm_5.unwrap() as f64, &tel, false)),
            },
            CAP_T: match value.CAP_T {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CAP_T", value.CAP_T.unwrap() as f64, &tel, false)),
            },
            DEM_LIM: match value.DEM_LIM {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("DEM_LIM", value.DEM_LIM.unwrap() as f64, &tel, false)),
            },
            LAG_LIM: match value.LAG_LIM {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("LAG_LIM", value.LAG_LIM.unwrap() as f64, &tel, false)),
            },
            SP: match value.SP {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("SP", value.SP.unwrap() as f64, &tel, false)),
            },
            CTRL_PNT: match value.CTRL_PNT {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CTRL_PNT", value.CTRL_PNT.unwrap() as f64, &tel, false)),
            },
            EMSTOP: match value.EMSTOP {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("EMSTOP", value.EMSTOP.unwrap() as f64, &tel, false)),
            },
            CP_A1: match value.CP_A1 {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CP_A1", value.CP_A1.unwrap() as f64, &tel, false)),
            },
            CP_A2: match value.CP_A2 {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CP_A2", value.CP_A2.unwrap() as f64, &tel, false)),
            },
            CAPA_T: match value.CAPA_T {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CAPA_T", value.CAPA_T.unwrap() as f64, &tel, false)),
            },
            DP_A: match value.DP_A {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("DP_A", value.DP_A.unwrap() as f64, &tel, false)),
            },
            SP_A: match value.SP_A {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("SP_A", value.SP_A.unwrap() as f64, &tel, false)),
            },
            SCT_A: match value.SCT_A {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("SCT_A", value.SCT_A.unwrap() as f64, &tel, false)),
            },
            SST_A: match value.SST_A {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("SST_A", value.SST_A.unwrap() as f64, &tel, false)),
            },
            CP_B1: match value.CP_B1 {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CP_B1", value.CP_B1.unwrap() as f64, &tel, false)),
            },
            CP_B2: match value.CP_B2 {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CP_B2", value.CP_B2.unwrap() as f64, &tel, false)),
            },
            CAPB_T: match value.CAPB_T {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CAPB_T", value.CAPB_T.unwrap() as f64, &tel, false)),
            },
            DP_B: match value.DP_B {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("DP_B", value.DP_B.unwrap() as f64, &tel, false)),
            },
            SP_B: match value.SP_B {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("SP_B", value.SP_B.unwrap() as f64, &tel, false)),
            },
            SCT_B: match value.SCT_B {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("SCT_B", value.SCT_B.unwrap() as f64, &tel, false)),
            },
            SST_B: match value.SST_B {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("SST_B", value.SST_B.unwrap() as f64, &tel, false)),
            },
            COND_LWT: match value.COND_LWT {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("COND_LWT", value.COND_LWT.unwrap() as f64, &tel, false)),
            },
            COND_EWT: match value.COND_EWT {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("COND_EWT", value.COND_EWT.unwrap() as f64, &tel, false)),
            },
            COOL_LWT: match value.COOL_LWT {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("COOL_LWT", value.COOL_LWT.unwrap() as f64, &tel, false)),
            },
            COOL_EWT: match value.COOL_EWT {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("COOL_EWT", value.COOL_EWT.unwrap() as f64, &tel, false)),
            },
            CPA1_OP: match value.CPA1_OP {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CPA1_OP", value.CPA1_OP.unwrap() as f64, &tel, false)),
            },
            CPA2_OP: match value.CPA2_OP {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CPA2_OP", value.CPA2_OP.unwrap() as f64, &tel, false)),
            },
            DOP_A1: match value.DOP_A1 {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("DOP_A1", value.DOP_A1.unwrap() as f64, &tel, false)),
            },
            DOP_A2: match value.DOP_A2 {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("DOP_A2", value.DOP_A2.unwrap() as f64, &tel, false)),
            },
            CPA1_DGT: match value.CPA1_DGT {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CPA1_DGT", value.CPA1_DGT.unwrap() as f64, &tel, false)),
            },
            CPA2_DGT: match value.CPA2_DGT {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CPA2_DGT", value.CPA2_DGT.unwrap() as f64, &tel, false)),
            },
            EXV_A: match value.EXV_A {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("EXV_A", value.EXV_A.unwrap() as f64, &tel, false)),
            },
            HR_CP_A1: match value.HR_CP_A1 {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("HR_CP_A1", value.HR_CP_A1.unwrap() as f64, &tel, false)),
            },
            HR_CP_A2: match value.HR_CP_A2 {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("HR_CP_A2", value.HR_CP_A2.unwrap() as f64, &tel, false)),
            },
            CPA1_TMP: match value.CPA1_TMP {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CPA1_TMP", value.CPA1_TMP.unwrap() as f64, &tel, false)),
            },
            CPA2_TMP: match value.CPA2_TMP {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CPA2_TMP", value.CPA2_TMP.unwrap() as f64, &tel, false)),
            },
            CPA1_CUR: match value.CPA1_CUR {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CPA1_CUR", value.CPA1_CUR.unwrap() as f64, &tel, false)),
            },
            CPA2_CUR: match value.CPA2_CUR {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CPA2_CUR", value.CPA2_CUR.unwrap() as f64, &tel, false)),
            },
            CPB1_OP: match value.CPB1_OP {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CPB1_OP", value.CPB1_OP.unwrap() as f64, &tel, false)),
            },
            CPB2_OP: match value.CPB2_OP {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CPB2_OP", value.CPB2_OP.unwrap() as f64, &tel, false)),
            },
            DOP_B1: match value.DOP_B1 {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("DOP_B1", value.DOP_B1.unwrap() as f64, &tel, false)),
            },
            DOP_B2: match value.DOP_B2 {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("DOP_B2", value.DOP_B2.unwrap() as f64, &tel, false)),
            },
            CPB1_DGT: match value.CPB1_DGT {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CPB1_DGT", value.CPB1_DGT.unwrap() as f64, &tel, false)),
            },
            CPB2_DGT: match value.CPB2_DGT {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CPB2_DGT", value.CPB2_DGT.unwrap() as f64, &tel, false)),
            },
            EXV_B: match value.EXV_B {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("EXV_B", value.EXV_B.unwrap() as f64, &tel, false)),
            },
            HR_CP_B1: match value.HR_CP_B1 {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("HR_CP_B1", value.HR_CP_B1.unwrap() as f64, &tel, false)),
            },
            HR_CP_B2: match value.HR_CP_B2 {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("HR_CP_B2", value.HR_CP_B2.unwrap() as f64, &tel, false)),
            },
            CPB1_TMP: match value.CPB1_TMP {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CPB1_TMP", value.CPB1_TMP.unwrap() as f64, &tel, false)),
            },
            CPB2_TMP: match value.CPB2_TMP {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CPB2_TMP", value.CPB2_TMP.unwrap() as f64, &tel, false)),
            },
            CPB1_CUR: match value.CPB1_CUR {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CPB1_CUR", value.CPB1_CUR.unwrap() as f64, &tel, false)),
            },
            CPB2_CUR: match value.CPB2_CUR {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CPB2_CUR", value.CPB2_CUR.unwrap() as f64, &tel, false)),
            },
            COND_SP: match value.COND_SP {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("COND_SP", value.COND_SP.unwrap() as f64, &tel, false)),
            },
            CHIL_OCC: match value.CHIL_OCC {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CHIL_OCC", value.CHIL_OCC.unwrap() as f64, &tel, false)),
            },
            STATUS: match value.STATUS {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("STATUS", value.STATUS.unwrap() as f64, &tel, false)),
            },
        };
        Ok(result)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DriChillerCarrierChangeParams {
    pub timestamp: NaiveDateTime,
    pub CHIL_S_S: Option<f64>,
    pub ALM: Option<f64>,
    pub EMSTOP: Option<f64>,
    pub STATUS: Option<f64>,
    pub CP_A1: Option<f64>,
    pub CP_A2: Option<f64>,
    pub CP_B1: Option<f64>,
    pub CP_B2: Option<f64>,
    pub CHIL_OCC: Option<f64>,
}

impl DriChillerCarrierChangeParams {
    pub fn new(timestamp: NaiveDateTime) -> Self {
        Self {
            timestamp,
            CHIL_S_S: None,
            ALM: None,
            EMSTOP: None,
            STATUS: None,
            CP_A1: None,
            CP_A2: None,
            CP_B1: None,
            CP_B2: None,
            CHIL_OCC: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TelemetryDriChillerCarrierXA<'a> {
    pub dev_id: Cow<'a, String>,
    pub timestamp: Cow<'a, String>,
    #[serde(rename="type")]
    pub dev_type: Cow<'a, String>,
    pub CAP_T: Option<i16>,
    pub CHIL_OCC: Option<i16>,
    pub CHIL_S_S: Option<i16>,
    pub COND_EWT: Option<i16>,
    pub COND_LWT: Option<i16>,
    pub COOL_EWT: Option<i16>,
    pub COOL_LWT: Option<i16>,
    pub CTRL_PNT: Option<i16>,
    pub CTRL_TYP: Option<i16>,
    pub DEM_LIM: Option<i16>,
    pub DP_A: Option<i16>,
    pub DP_B: Option<i16>,
    pub EMSTOP: Option<i16>,
    pub HR_CP_A: Option<i64>,
    pub HR_CP_B: Option<i64>,
    pub HR_MACH: Option<i64>,
    pub HR_MACH_B: Option<i64>,
    pub OAT: Option<i16>,
    pub OP_A: Option<i16>,
    pub OP_B: Option<i16>,
    pub SCT_A: Option<i16>,
    pub SCT_B: Option<i16>,
    pub SLC_HM: Option<i16>,
    pub SLT_A: Option<i16>,
    pub SLT_B: Option<i16>,
    pub SP: Option<i16>,
    pub SP_A: Option<i16>,
    pub SP_B: Option<i16>,
    pub SP_OCC: Option<i16>,
    pub SST_A: Option<i16>,
    pub SST_B: Option<i16>,
    pub STATUS: Option<i16>,
    pub formulas: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DriChillerCarrierXATelemetry {
    pub timestamp: NaiveDateTime,
    pub CAP_T: Option<f64>,
    pub CHIL_OCC: Option<f64>,
    pub CHIL_S_S: Option<f64>,
    pub COND_EWT: Option<f64>,
    pub COND_LWT: Option<f64>,
    pub COOL_EWT: Option<f64>,
    pub COOL_LWT: Option<f64>,
    pub CTRL_PNT: Option<f64>,
    pub CTRL_TYP: Option<f64>,
    pub DEM_LIM: Option<f64>,
    pub DP_A: Option<f64>,
    pub DP_B: Option<f64>,
    pub EMSTOP: Option<f64>,
    pub HR_CP_A: Option<f64>,
    pub HR_CP_B: Option<f64>,
    pub HR_MACH: Option<f64>,
    pub HR_MACH_B: Option<f64>,
    pub OAT: Option<f64>,
    pub OP_A: Option<f64>,
    pub OP_B: Option<f64>,
    pub SCT_A: Option<f64>,
    pub SCT_B: Option<f64>,
    pub SLC_HM: Option<f64>,
    pub SLT_A: Option<f64>,
    pub SLT_B: Option<f64>,
    pub SP: Option<f64>,
    pub SP_A: Option<f64>,
    pub SP_B: Option<f64>,
    pub SP_OCC: Option<f64>,
    pub SST_A: Option<f64>,
    pub SST_B: Option<f64>,
    pub STATUS: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DriChillerCarrierXAHvarTelemetry {
    pub timestamp: NaiveDateTime,
    pub GENUNIT_UI: Option<f64>,
    pub CTRL_TYP: Option<f64>,
    pub STATUS: Option<f64>,
    pub ALM: Option<f64>,
    pub SP_OCC: Option<f64>,
    pub CHIL_S_S: Option<f64>,
    pub CHIL_OCC: Option<f64>,
    pub CAP_T: Option<f64>,
    pub DEM_LIM: Option<f64>,
    pub TOT_CURR: Option<f64>,
    pub CTRL_PNT: Option<f64>,
    pub OAT: Option<f64>,
    pub COOL_EWT: Option<f64>,
    pub COOL_LWT: Option<f64>,
    pub EMSTOP: Option<f64>,
    pub CIRCA_AN_UI: Option<f64>,
    pub CAPA_T: Option<f64>,
    pub DP_A: Option<f64>,
    pub SP_A: Option<f64>,
    pub ECON_P_A: Option<f64>,
    pub OP_A: Option<f64>,
    pub DOP_A: Option<f64>,
    pub CURREN_A: Option<f64>,
    pub CP_TMP_A: Option<f64>,
    pub DGT_A: Option<f64>,
    pub ECO_TP_A: Option<f64>,
    pub SCT_A: Option<f64>,
    pub SST_A: Option<f64>,
    pub SST_B: Option<f64>,
    pub SUCT_T_A: Option<f64>,
    pub EXV_A: Option<f64>,
    pub CIRCB_AN_UI: Option<f64>,
    pub CAPB_T: Option<f64>,
    pub DP_B: Option<f64>,
    pub SP_B: Option<f64>,
    pub ECON_P_B: Option<f64>,
    pub OP_B: Option<f64>,
    pub DOP_B: Option<f64>,
    pub CURREN_B: Option<f64>,
    pub CP_TMP_B: Option<f64>,
    pub DGT_B: Option<f64>,
    pub ECO_TP_B: Option<f64>,
    pub SCT_B: Option<f64>,
    pub SUCT_T_B: Option<f64>,
    pub EXV_B: Option<f64>,
    pub CIRCC_AN_UI: Option<f64>,
    pub CAPC_T: Option<f64>,
    pub DP_C: Option<f64>,
    pub SP_C: Option<f64>,
    pub ECON_P_C: Option<f64>,
    pub OP_C: Option<f64>,
    pub DOP_C: Option<f64>,
    pub CURREN_C: Option<f64>,
    pub CP_TMP_C: Option<f64>,
    pub DGT_C: Option<f64>,
    pub ECO_TP_C: Option<f64>,
    pub SCT_C: Option<f64>,
    pub SST_C: Option<f64>,
    pub SUCT_T_C: Option<f64>,
    pub EXV_C: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TelemetryDriChillerCarrierXAHvar<'a> {
    pub dev_id: Cow<'a, String>,
    pub timestamp: Cow<'a, String>,
    #[serde(rename="type")]
    pub dev_type: Cow<'a, String>,
    pub CAP_T: Option<i16>,
    pub CHIL_OCC: Option<i16>,
    pub CHIL_S_S: Option<i16>,
    pub COOL_EWT: Option<i16>,
    pub COOL_LWT: Option<i16>,
    pub CTRL_PNT: Option<i16>,
    pub CTRL_TYP: Option<i16>,
    pub DEM_LIM: Option<i16>,
    pub DP_A: Option<i16>,
    pub DP_B: Option<i16>,
    pub EMSTOP: Option<i16>,
    pub OAT: Option<i16>,
    pub OP_A: Option<i16>,
    pub OP_B: Option<i16>,
    pub SCT_A: Option<i16>,
    pub SCT_B: Option<i16>,
    pub SP_A: Option<i16>,
    pub SP_B: Option<i16>,
    pub SP_OCC: Option<i16>,
    pub SST_A: Option<i16>,
    pub SST_B: Option<i16>,
    pub STATUS: Option<i16>,
    pub GENUNIT_UI: Option<i16>,
    pub ALM: Option<i16>,
    pub TOT_CURR: Option<i16>,
    pub CIRCA_AN_UI: Option<i16>,
    pub CAPA_T: Option<i16>,
    pub ECON_P_A: Option<i16>,
    pub DOP_A: Option<i16>,
    pub CURREN_A: Option<i16>,
    pub CP_TMP_A: Option<i16>,
    pub DGT_A: Option<i16>,
    pub ECO_TP_A: Option<i16>,
    pub SUCT_T_A: Option<i16>,
    pub EXV_A: Option<i16>,
    pub CIRCB_AN_UI: Option<i16>,
    pub CAPB_T: Option<i16>,
    pub ECON_P_B: Option<i16>,
    pub DOP_B: Option<i16>,
    pub CURREN_B: Option<i16>,
    pub CP_TMP_B: Option<i16>,
    pub DGT_B: Option<i16>,
    pub ECO_TP_B: Option<i16>,
    pub SUCT_T_B: Option<i16>,
    pub EXV_B: Option<i16>,
    pub CIRCC_AN_UI: Option<i16>,
    pub CAPC_T: Option<i16>,
    pub DP_C: Option<i16>,
    pub SP_C: Option<i16>,
    pub ECON_P_C: Option<i16>,
    pub OP_C: Option<i16>,
    pub DOP_C: Option<i16>,
    pub CURREN_C: Option<i16>,
    pub CP_TMP_C: Option<i16>,
    pub DGT_C: Option<i16>,
    pub ECO_TP_C: Option<i16>,
    pub SCT_C: Option<i16>,
    pub SST_C: Option<i16>,
    pub SUCT_T_C: Option<i16>,
    pub EXV_C: Option<i16>,
    pub formulas: Option<HashMap<String, String>>,
}


impl DriChillerCarrierXAHvarTelemetry {
    pub fn set_field_average(&mut self, field: &str, value: f64) {
        match field {
            "GENUNIT_UI" => self.GENUNIT_UI = Some(value),
            "CAP_T" => self.CAP_T = Some(value),
            "TOT_CURR" => self.TOT_CURR = Some(value),
            "CTRL_PNT" => self.CTRL_PNT = Some(value),
            "OAT" => self.OAT = Some(value),
            "COOL_EWT" => self.COOL_EWT = Some(value),
            "COOL_LWT" => self.COOL_LWT = Some(value),
            "CIRCA_AN_UI" => self.CIRCA_AN_UI = Some(value),
            "CAPA_T" => self.CAPA_T = Some(value),
            "DP_A" => self.DP_A = Some(value),
            "SP_A" => self.SP_A = Some(value),
            "ECON_P_A" => self.ECON_P_A = Some(value),
            "OP_A" => self.OP_A = Some(value),
            "DOP_A" => self.DOP_A = Some(value),
            "CURREN_A" => self.CURREN_A = Some(value),
            "CP_TMP_A" => self.CP_TMP_A = Some(value),
            "DGT_A" => self.DGT_A = Some(value),
            "ECO_TP_A" => self.ECO_TP_A = Some(value),
            "SCT_A" => self.SCT_A = Some(value),
            "SST_A" => self.SST_A = Some(value),
            "SST_B" => self.SST_B = Some(value),
            "SUCT_T_A" => self.SUCT_T_A = Some(value),
            "EXV_A" => self.EXV_A = Some(value),
            "CIRCB_AN_UI" => self.CIRCB_AN_UI = Some(value),
            "CAPB_T" => self.CAPB_T = Some(value),
            "DP_B" => self.DP_B = Some(value),
            "SP_B" => self.SP_B = Some(value),
            "ECON_P_B" => self.ECON_P_B = Some(value),
            "OP_B" => self.OP_B = Some(value),
            "DOP_B" => self.DOP_B = Some(value),
            "CURREN_B" => self.CURREN_B = Some(value),
            "CP_TMP_B" => self.CP_TMP_B = Some(value),
            "DGT_B" => self.DGT_B = Some(value),
            "ECO_TP_B" => self.ECO_TP_B = Some(value),
            "SCT_B" => self.SCT_B = Some(value),
            "SUCT_T_B" => self.SUCT_T_B = Some(value),
            "EXV_B" => self.EXV_B = Some(value),
            "CIRCC_AN_UI" => self.CIRCC_AN_UI = Some(value),
            "CAPC_T" => self.CAPC_T = Some(value),
            "DP_C" => self.DP_C = Some(value),
            "SP_C" => self.SP_C = Some(value),
            "ECON_P_C" => self.ECON_P_C = Some(value),
            "OP_C" => self.OP_C = Some(value),
            "DOP_C" => self.DOP_C = Some(value),
            "CURREN_C" => self.CURREN_C = Some(value),
            "CP_TMP_C" => self.CP_TMP_C = Some(value),
            "DGT_C" => self.DGT_C = Some(value),
            "ECO_TP_C" => self.ECO_TP_C = Some(value),
            "SCT_C" => self.SCT_C = Some(value),
            "SST_C" => self.SST_C = Some(value),
            "SUCT_T_C" => self.SUCT_T_C = Some(value),
            "EXV_C" => self.EXV_C = Some(value),
            _ => (),
        }
    }
    
    pub fn new(timestamp: NaiveDateTime) -> Self {
        Self {
            timestamp,
            GENUNIT_UI: None,
            CTRL_TYP: None,
            STATUS: None,
            ALM: None,
            SP_OCC: None,
            CHIL_S_S: None,
            CHIL_OCC: None,
            CAP_T: None,
            DEM_LIM: None,
            TOT_CURR: None,
            CTRL_PNT: None,
            OAT: None,
            COOL_EWT: None,
            COOL_LWT: None,
            EMSTOP: None,
            CIRCA_AN_UI: None,
            CAPA_T: None,
            DP_A: None,
            SP_A: None,
            ECON_P_A: None,
            OP_A: None,
            DOP_A: None,
            CURREN_A: None,
            CP_TMP_A: None,
            DGT_A: None,
            ECO_TP_A: None,
            SCT_A: None,
            SST_A: None,
            SST_B: None,
            SUCT_T_A: None,
            EXV_A: None,
            CIRCB_AN_UI: None,
            CAPB_T: None,
            DP_B: None,
            SP_B: None,
            ECON_P_B: None,
            OP_B: None,
            DOP_B: None,
            CURREN_B: None,
            CP_TMP_B: None,
            DGT_B: None,
            ECO_TP_B: None,
            SCT_B: None,
            SUCT_T_B: None,
            EXV_B: None,
            CIRCC_AN_UI: None,
            CAPC_T: None,
            DP_C: None,
            SP_C: None,
            ECON_P_C: None,
            OP_C: None,
            DOP_C: None,
            CURREN_C: None,
            CP_TMP_C: None,
            DGT_C: None,
            ECO_TP_C: None,
            SCT_C: None,
            SST_C: None,
            SUCT_T_C: None,
            EXV_C: None,
            
        }
    }
}

impl<'a> TryFrom<TelemetryDriChillerCarrierXAHvar<'a>> for DriChillerCarrierXAHvarTelemetry {
    type Error = String;
    fn try_from(value: TelemetryDriChillerCarrierXAHvar) -> Result<DriChillerCarrierXAHvarTelemetry, String> {
        let tel = json!(value);
        let result = DriChillerCarrierXAHvarTelemetry {
            timestamp: NaiveDateTime::parse_from_str(value.timestamp.as_ref(), "%Y-%m-%dT%H:%M:%S")
            .map_err(|e| e.to_string())?,
            GENUNIT_UI: match value.GENUNIT_UI {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("GENUNIT_UI", value.GENUNIT_UI.unwrap() as f64, &tel, false)),
            },
            SUCT_T_B: match value.SUCT_T_B {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("SUCT_T_B", value.SUCT_T_B.unwrap() as f64, &tel, false)),
            },
            SUCT_T_C: match value.SUCT_T_C {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("SUCT_T_C", value.SUCT_T_C.unwrap() as f64, &tel, false)),
            },
            TOT_CURR: match value.TOT_CURR {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("TOT_CURR", value.TOT_CURR.unwrap() as f64, &tel, false)),
            },
            SP_C: match value.SP_C {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("SP_C", value.SP_C.unwrap() as f64, &tel, false)),
            },
            SST_C: match value.SST_C {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("SST_C", value.SST_C.unwrap() as f64, &tel, false)),
            },
            SUCT_T_A: match value.SUCT_T_A {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("SUCT_T_A", value.SUCT_T_A.unwrap() as f64, &tel, false)),
            },
            EXV_C: match value.EXV_C {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("EXV_C", value.EXV_C.unwrap() as f64, &tel, false)),
            },
            OP_C: match value.OP_C {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("OP_C", value.OP_C.unwrap() as f64, &tel, false)),
            },
            SCT_C: match value.SCT_C {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("SCT_C", value.SCT_C.unwrap() as f64, &tel, false)),
            },
            ECO_TP_C: match value.ECO_TP_C {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("ECO_TP_C", value.ECO_TP_C.unwrap() as f64, &tel, false)),
            },
            EXV_A: match value.EXV_A {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("EXV_A", value.EXV_A.unwrap() as f64, &tel, false)),
            },
            EXV_B: match value.EXV_B {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("EXV_B", value.EXV_B.unwrap() as f64, &tel, false)),
            },
            ECON_P_C: match value.ECON_P_C {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("ECON_P_C", value.ECON_P_C.unwrap() as f64, &tel, false)),
            },
            ECO_TP_A: match value.ECO_TP_A {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("ECO_TP_A", value.ECO_TP_A.unwrap() as f64, &tel, false)),
            },
            ECO_TP_B: match value.ECO_TP_B {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("ECO_TP_B", value.ECO_TP_B.unwrap() as f64, &tel, false)),
            },
            DP_C: match value.DP_C {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("DP_C", value.DP_C.unwrap() as f64, &tel, false)),
            },
            ECON_P_A: match value.ECON_P_A {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("ECON_P_A", value.ECON_P_A.unwrap() as f64, &tel, false)),
            },
            ECON_P_B: match value.ECON_P_B {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("ECON_P_B", value.ECON_P_B.unwrap() as f64, &tel, false)),
            },
            DOP_A: match value.DOP_A {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("DOP_A", value.DOP_A.unwrap() as f64, &tel, false)),
            },
            DOP_B: match value.DOP_B {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("DOP_B", value.DOP_B.unwrap() as f64, &tel, false)),
            },
            DOP_C: match value.DOP_C {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("DOP_C", value.DOP_C.unwrap() as f64, &tel, false)),
            },
            DGT_A: match value.DGT_A {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("DGT_A", value.DGT_A.unwrap() as f64, &tel, false)),
            },
            DGT_B: match value.DGT_B {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("DGT_B", value.DGT_B.unwrap() as f64, &tel, false)),
            },
            DGT_C: match value.DGT_C {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("DGT_C", value.DGT_C.unwrap() as f64, &tel, false)),
            },
            CURREN_A: match value.CURREN_A {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CURREN_A", value.CURREN_A.unwrap() as f64, &tel, false)),
            },
            CURREN_B: match value.CURREN_B {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CURREN_B", value.CURREN_B.unwrap() as f64, &tel, false)),
            },
            CURREN_C: match value.CURREN_C {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CURREN_C", value.CURREN_C.unwrap() as f64, &tel, false)),
            },
            CP_TMP_B: match value.CP_TMP_B {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CP_TMP_B", value.CP_TMP_B.unwrap() as f64, &tel, false)),
            },
            CP_TMP_A: match value.CP_TMP_A {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CP_TMP_A", value.CP_TMP_A.unwrap() as f64, &tel, false)),
            },
            CP_TMP_C: match value.CP_TMP_C {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CP_TMP_C", value.CP_TMP_C.unwrap() as f64, &tel, false)),
            },
            CIRCA_AN_UI: match value.CIRCA_AN_UI {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CIRCA_AN_UI", value.CIRCA_AN_UI.unwrap() as f64, &tel, false)),
            },
            CIRCB_AN_UI: match value.CIRCB_AN_UI {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CIRCB_AN_UI", value.CIRCB_AN_UI.unwrap() as f64, &tel, false)),
            },
            CIRCC_AN_UI: match value.CIRCC_AN_UI {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CIRCC_AN_UI", value.CIRCC_AN_UI.unwrap() as f64, &tel, false)),
            },
            CTRL_TYP: match value.CTRL_TYP {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CTRL_TYP", value.CTRL_TYP.unwrap() as f64, &tel, false)),
            },
            CAPA_T: match value.CAPA_T {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CAPA_T", value.CAPA_T.unwrap() as f64, &tel, false)),
            },
            CAPB_T: match value.CAPB_T {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CAPB_T", value.CAPB_T.unwrap() as f64, &tel, false)),
            },
            CAPC_T: match value.CAPC_T {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CAPC_T", value.CAPC_T.unwrap() as f64, &tel, false)),
            },
            STATUS: match value.STATUS {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("STATUS", value.STATUS.unwrap() as f64, &tel, false)),
            },
            ALM: match value.ALM {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("ALM", value.ALM.unwrap() as f64, &tel, false)),
            },
            SP_OCC: match value.SP_OCC {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("SP_OCC", value.SP_OCC.unwrap() as f64, &tel, false)),
            },
            CHIL_S_S: match value.CHIL_S_S {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CHIL_S_S", value.CHIL_S_S.unwrap() as f64, &tel, false)),
            },
            CHIL_OCC: match value.CHIL_OCC {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CHIL_OCC", value.CHIL_OCC.unwrap() as f64, &tel, false)),
            },
            CAP_T: match value.CAP_T {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CAP_T", value.CAP_T.unwrap() as f64, &tel, false)),
            },
            COOL_EWT: match value.COOL_EWT {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("COOL_EWT", value.COOL_EWT.unwrap() as f64, &tel, false)),
            },
            COOL_LWT: match value.COOL_LWT {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("COOL_LWT", value.COOL_LWT.unwrap() as f64, &tel, false)),
            },
            CTRL_PNT: match value.CTRL_PNT {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CTRL_PNT", value.CTRL_PNT.unwrap() as f64, &tel, false)),
            },
            DEM_LIM: match value.DEM_LIM {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("DEM_LIM", value.DEM_LIM.unwrap() as f64, &tel, false)),
            },
            DP_A: match value.DP_A {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("DP_A", value.DP_A.unwrap() as f64, &tel, false)),
            },
            DP_B: match value.DP_B {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("DP_B", value.DP_B.unwrap() as f64, &tel, false)),
            },
            EMSTOP: match value.EMSTOP {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("EMSTOP", value.EMSTOP.unwrap() as f64, &tel, false)),
            },
            OAT: match value.OAT {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("OAT", value.OAT.unwrap() as f64, &tel, false)),
            },
            OP_A: match value.OP_A {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("OP_A", value.OP_A.unwrap() as f64, &tel, false)),
            },
            OP_B: match value.OP_B {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("OP_B", value.OP_B.unwrap() as f64, &tel, false)),
            },
            SCT_A: match value.SCT_A {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("SCT_A", value.SCT_A.unwrap() as f64, &tel, false)),
            },
            SCT_B: match value.SCT_B {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("SCT_B", value.SCT_B.unwrap() as f64, &tel, false)),
            },
            SP_A: match value.SP_A {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("SP_A", value.SP_A.unwrap() as f64, &tel, false)),
            },
            SP_B: match value.SP_B {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("SP_B", value.SP_B.unwrap() as f64, &tel, false)),
            },
            SST_A: match value.SST_A {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("SST_A", value.SST_A.unwrap() as f64, &tel, false)),
            },
            SST_B: match value.SST_B {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("SST_B", value.SST_B.unwrap() as f64, &tel, false)),
            },
        };
        Ok(result)
    }
}

impl<'a> TryFrom<TelemetryDriChillerCarrierXA<'a>> for DriChillerCarrierXATelemetry {
    type Error = String;
    fn try_from(value: TelemetryDriChillerCarrierXA) -> Result<DriChillerCarrierXATelemetry, String> {
        let tel = json!(value);

        let result = DriChillerCarrierXATelemetry {
            timestamp: NaiveDateTime::parse_from_str(value.timestamp.as_ref(), "%Y-%m-%dT%H:%M:%S")
            .map_err(|e| e.to_string())?,
            CAP_T: match value.CAP_T {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CAP_T", value.CAP_T.unwrap() as f64, &tel, false)),
            },
            CHIL_OCC: match value.CHIL_OCC {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CHIL_OCC", value.CHIL_OCC.unwrap() as f64, &tel, false)),
            },
            CHIL_S_S: match value.CHIL_S_S {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CHIL_S_S", value.CHIL_S_S.unwrap() as f64, &tel, false)),
            },
            COND_EWT: match value.COND_EWT {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("COND_EWT", value.COND_EWT.unwrap() as f64, &tel, false)),
            },
            COND_LWT: match value.COND_LWT {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("COND_LWT", value.COND_LWT.unwrap() as f64, &tel, false)),
            },
            COOL_EWT: match value.COOL_EWT {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("COOL_EWT", value.COOL_EWT.unwrap() as f64, &tel, false)),
            },
            COOL_LWT: match value.COOL_LWT {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("COOL_LWT", value.COOL_LWT.unwrap() as f64, &tel, false)),
            },
            CTRL_PNT: match value.CTRL_PNT {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CTRL_PNT", value.CTRL_PNT.unwrap() as f64, &tel, false)),
            },
            CTRL_TYP: match value.CTRL_TYP {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("CTRL_TYP", value.CTRL_TYP.unwrap() as f64, &tel, false)),
            },
            DEM_LIM: match value.DEM_LIM {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("DEM_LIM", value.DEM_LIM.unwrap() as f64, &tel, false)),
            },
            DP_A: match value.DP_A {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("DP_A", value.DP_A.unwrap() as f64, &tel, false)),
            },
            DP_B: match value.DP_B {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("DP_B", value.DP_B.unwrap() as f64, &tel, false)),
            },
            EMSTOP: match value.EMSTOP {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("EMSTOP", value.EMSTOP.unwrap() as f64, &tel, false)),
            },
            HR_CP_A: match value.HR_CP_A {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("HR_CP_A", value.HR_CP_A.unwrap() as f64, &tel, false)),
            },
            HR_CP_B: match value.HR_CP_B {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("HR_CP_B", value.HR_CP_B.unwrap() as f64, &tel, false)),
            },
            HR_MACH: match value.HR_MACH {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("HR_MACH", value.HR_MACH.unwrap() as f64, &tel, false)),
            },
            HR_MACH_B: match value.HR_MACH_B {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("HR_MACH_B", value.HR_MACH_B.unwrap() as f64, &tel, false)),
            },
            OAT: match value.OAT {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("OAT", value.OAT.unwrap() as f64, &tel, false)),
            },
            OP_A: match value.OP_A {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("OP_A", value.OP_A.unwrap() as f64, &tel, false)),
            },
            OP_B: match value.OP_B {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("OP_B", value.OP_B.unwrap() as f64, &tel, false)),
            },
            SCT_A: match value.SCT_A {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("SCT_A", value.SCT_A.unwrap() as f64, &tel, false)),
            },
            SCT_B: match value.SCT_B {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("SCT_B", value.SCT_B.unwrap() as f64, &tel, false)),
            },
            SLC_HM: match value.SLC_HM {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("SLC_HM", value.SLC_HM.unwrap() as f64, &tel, false)),
            },
            SLT_A: match value.SLT_A {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("SLT_A", value.SLT_A.unwrap() as f64, &tel, false)),
            },
            SLT_B: match value.SLT_B {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("SLT_B", value.SLT_B.unwrap() as f64, &tel, false)),
            },
            SP: match value.SP {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("SP", value.SP.unwrap() as f64, &tel, false)),
            },
            SP_A: match value.SP_A {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("SP_A", value.SP_A.unwrap() as f64, &tel, false)),
            },
            SP_B: match value.SP_B {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("SP_B", value.SP_B.unwrap() as f64, &tel, false)),
            },
            SP_OCC: match value.SP_OCC {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("SP_OCC", value.SP_OCC.unwrap() as f64, &tel, false)),
            },
            SST_A: match value.SST_A {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("SST_A", value.SST_A.unwrap() as f64, &tel, false)),
            },
            SST_B: match value.SST_B {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("SST_B", value.SST_B.unwrap() as f64, &tel, false)),
            },
            STATUS: match value.STATUS {
                None => None,
                Some(-1) => None,
                _ => Some(calculate_formulas("STATUS", value.STATUS.unwrap() as f64, &tel, false)),
            },
        };
        Ok(result)
    }
}

impl DriChillerCarrierXATelemetry {
    pub fn set_field_average(&mut self, field: &str, value: f64) {
        match field {
            "CAP_T" => self.CAP_T = Some(value),
            "COND_EWT" => self.COND_EWT = Some(value),
            "COND_LWT" => self.COND_LWT = Some(value),
            "COOL_EWT" => self.COOL_EWT = Some(value),
            "COOL_LWT" => self.COOL_LWT = Some(value),
            "CTRL_PNT" => self.CTRL_PNT = Some(value),
            "DP_A" => self.DP_A = Some(value),
            "DP_B" => self.DP_B = Some(value),
            "HR_CP_A" => self.HR_CP_A = Some(value),
            "HR_CP_B" => self.HR_CP_B = Some(value),
            "HR_MACH" => self.HR_MACH = Some(value),
            "HR_MACH_B" => self.HR_MACH_B = Some(value),
            "OAT" => self.OAT = Some(value),
            "OP_A" => self.OP_A = Some(value),
            "OP_B" => self.OP_B = Some(value),
            "SCT_A" => self.SCT_A = Some(value),
            "SCT_B" => self.SCT_B = Some(value),
            "SLT_A" => self.SLT_A = Some(value),
            "SLT_B" => self.SLT_B = Some(value),
            "SP" => self.SP = Some(value),
            "SP_A" => self.SP_A = Some(value),
            "SP_B" => self.SP_B = Some(value),
            "SST_A" => self.SST_A = Some(value),
            "SST_B" => self.SST_B = Some(value),
            _ => (),
        }
    }
    
    pub fn new(timestamp: NaiveDateTime) -> Self {
        Self {
            timestamp,
            CAP_T: None,
            CHIL_OCC: None,
            CHIL_S_S: None,
            COND_EWT: None,
            COND_LWT: None,
            COOL_EWT: None,
            COOL_LWT: None,
            CTRL_PNT: None,
            CTRL_TYP: None,
            DEM_LIM: None,
            DP_A: None,
            DP_B: None,
            EMSTOP: None,
            HR_CP_A: None,
            HR_CP_B: None,
            HR_MACH: None,
            HR_MACH_B: None,
            OAT: None,
            OP_A: None,
            OP_B: None,
            SCT_A: None,
            SCT_B: None,
            SLC_HM: None,
            SLT_A: None,
            SLT_B: None,
            SP: None,
            SP_A: None,
            SP_B: None,
            SP_OCC: None,
            SST_A: None,
            SST_B: None,
            STATUS: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DriChillerCarrierXAChangeParams {
    pub timestamp: NaiveDateTime,
    pub STATUS: Option<f64>,
    pub CHIL_S_S: Option<f64>,
    pub CHIL_OCC: Option<f64>,
    pub CTRL_TYP: Option<f64>,
    pub SLC_HM: Option<f64>,
    pub DEM_LIM: Option<f64>,
    pub SP_OCC: Option<f64>,
    pub EMSTOP: Option<f64>,
}

impl DriChillerCarrierXAChangeParams {
    pub fn new(timestamp: NaiveDateTime) -> Self {
        Self {
            timestamp,
            STATUS: None,
            CHIL_S_S: None,
            CHIL_OCC: None,
            CTRL_TYP: None,
            SLC_HM: None,
            DEM_LIM: None,
            SP_OCC: None,
            EMSTOP: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DriChillerCarrierXAHvarChangeParams {
    pub timestamp: NaiveDateTime,
    pub STATUS: Option<f64>,
    pub CTRL_TYP: Option<f64>,
    pub ALM: Option<f64>,
    pub SP_OCC: Option<f64>,
    pub CHIL_S_S: Option<f64>,
    pub CHIL_OCC: Option<f64>,
    pub DEM_LIM: Option<f64>,
    pub EMSTOP: Option<f64>,
}

impl DriChillerCarrierXAHvarChangeParams {
    pub fn new(timestamp: NaiveDateTime) -> Self {
        Self {
            timestamp,
            STATUS: None,
            CTRL_TYP: None,
            ALM: None,
            SP_OCC: None,
            CHIL_S_S: None,
            CHIL_OCC: None,
            DEM_LIM: None,
            EMSTOP: None,
        }
    }
}
