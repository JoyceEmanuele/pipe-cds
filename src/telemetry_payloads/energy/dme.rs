use serde_json::Value;
use serde_with::{serde_as, DeserializeAs, SerializeAs};
use std::{borrow::Cow, collections::HashMap};
use serde::{
    de::{Error, Unexpected},
    Deserialize, Deserializer, Serialize,
};

use crate::telemetry_payloads::dri_telemetry::HwInfoDRI;

use super::padronized::PadronizedEnergyTelemetry;

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TelemetryDME<'a> {
    pub dev_id: Cow<'a, String>,
    pub timestamp: Cow<'a, String>,
    #[serde(rename="type")]
    pub dev_type: Option<Cow<'a, String>>,
    pub v_a: Option<f64>,
    pub v_b: Option<f64>,
    pub v_c: Option<f64>,
    pub v_ab: Option<f64>,
    pub v_bc: Option<f64>,
    pub v_ca: Option<f64>,
    pub i_a: Option<f64>,
    pub i_b: Option<f64>,
    pub i_c: Option<f64>,
    pub pot_at_a: Option<f64>,
    pub pot_at_b: Option<f64>,
    pub pot_at_c: Option<f64>,
    pub pot_ap_a: Option<f64>,
    pub pot_ap_b: Option<f64>,
    pub pot_ap_c: Option<f64>,
    pub pot_re_a: Option<f64>,
    pub pot_re_b: Option<f64>,
    pub pot_re_c: Option<f64>,
    pub v_tri_ln: Option<f64>,
    pub v_tri_ll: Option<f64>,
    pub pot_at_tri: Option<f64>,
    pub pot_ap_tri: Option<f64>,
    pub pot_re_tri: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<VerifyStringOrf64>")]
    pub en_at_tri: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<VerifyStringOrf64>")]
    pub en_re_tri: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<VerifyStringOrf64>")]
    pub en_ap_tri: Option<f64>,
    pub fp_a: Option<f64>,
    pub fp_b: Option<f64>,
    pub fp_c: Option<f64>,
    pub fp: Option<f64>,
    pub freq: Option<f64>,
    pub demanda: Option<f64>,
    pub demanda_at: Option<f64>,
    pub demanda_ap: Option<f64>,
    pub demanda_med_at: Option<f64>,
    pub erro: Option<f64>,

    pub CMN0: Option<f64>,
    pub CMN1: Option<f64>,
    pub CMN2: Option<f64>,
    pub CMN3: Option<f64>,
    pub CMN4: Option<f64>,
    pub CMN5: Option<f64>,
    pub CMN6: Option<f64>,
    pub CMN7: Option<f64>,
    pub CMN8: Option<f64>,
    pub CMN9: Option<f64>,
    pub CMN10: Option<f64>,
    pub CMN11: Option<f64>,
    pub CMN12: Option<f64>,
    pub CMN13: Option<f64>,
    pub CMN14: Option<f64>,
    pub CMN15: Option<f64>,
    pub CMN16: Option<f64>,
    pub CMN17: Option<f64>,
    pub CMN18: Option<f64>,
    pub CMN19: Option<f64>,
    pub CMN20: Option<f64>,
    pub CMN21: Option<f64>,
    pub CMN22: Option<f64>,
    pub CMN23: Option<f64>,
    pub CMN24: Option<f64>,
    pub CMN25: Option<f64>,
    pub CMN26: Option<f64>,
    pub CMN27: Option<f64>,
    pub CMN28: Option<f64>,
    pub CMN29: Option<f64>,
    pub CMN30: Option<f64>,
    pub CMN31: Option<f64>,
    pub CMN32: Option<f64>,
    pub CMN33: Option<f64>,
    pub CMN34: Option<f64>,
    pub CMN35: Option<f64>,
    pub CMN36: Option<f64>,
    pub CMN37: Option<f64>,
    pub CMN38: Option<f64>,
    pub CMN39: Option<f64>,
    pub CMN40: Option<f64>,
    pub CMN41: Option<f64>,
    pub formulas: Option<HashMap<String, String>>,
}

fn f64_from_str<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    match Value::deserialize(deserializer) {
        Ok(Value::Number(result)) => {
            result.as_f64().ok_or_else(||Error::invalid_type(Unexpected::Other(&result.to_string()),& "Tipo incorreto"))
        },

        Ok(Value::String(result)) => {
            result.parse::<f64>()
            .map_err(|e|Error::invalid_value(Unexpected::Str(&result), &"Float em String"))
        },
        Ok(wrong_value) => {
            Err(Error::invalid_type(Unexpected::Other(&wrong_value.to_string()), &"Tipo não adequado"))
        }
        Err(err) => {
            print!("{err}");
            Err(err)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum VerifyStringOrf64 {
    Temp1(Option<String>),
    Temp2(Option<f64>)
 }

impl SerializeAs<f64> for VerifyStringOrf64 {
  fn serialize_as<S>(source: &f64, serializer: S) -> Result<S::Ok, S::Error>
  where
          S: serde::Serializer {
      serializer.serialize_f64(*source)
  }
}

impl<'de> DeserializeAs<'de, f64> for VerifyStringOrf64 {
  fn deserialize_as<D>(deserializer: D) -> Result<f64, D::Error>
  where
        D: Deserializer<'de> {
        f64_from_str(deserializer)
  }
}
