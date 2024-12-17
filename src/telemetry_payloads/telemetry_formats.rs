use chrono::{NaiveDateTime, Duration};
use serde::{
    de::{Error, Unexpected},
    Deserialize, Deserializer, Serialize,
};
use serde_with::{serde_as, DeserializeAs, SerializeAs};
use serde_json::Value;

const fn always5() -> i64 {
    5
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TelemetryDMA {
    pub timestamp: String,
    pub pulses: Option<i32>,
    pub mode: Option<String>,
    pub operation_mode: Option<i16>,
    pub dev_id: String,
    pub samplingTime: Option<i16>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TelemetryPackDMA {
    pub timestamp: String,
    pub dev_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pulses: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_mode: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub samplingTime: Option<i16>,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct TelemetryPackDUT_v2 {
    pub timestamp: NaiveDateTime,
    #[serde(default = "always5")]
    pub samplingTime: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<Vec<Option<VerifyStringOrf64>>>")]
    pub Temperature: Option<Vec<Option<f64>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub Temperature_1: Option<Vec<Option<f64>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<Vec<Option<VerifyStringOrf64>>>")]
    pub Tmp: Option<Vec<Option<f64>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub Humidity: Option<Vec<Option<f64>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eCO2: Option<Vec<Option<i16>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_eCO2: Option<Vec<Option<i16>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "TVOC")]
    pub tvoc: Option<Vec<Option<i16>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub State: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub Mode: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TelemetryDUT_v3 {
    pub timestamp: NaiveDateTime,
    pub Temp: Option<f64>,
    pub Temp1: Option<f64>,
    pub Tmp: Option<f64>,
    pub Hum: Option<f64>,
    pub eCO2: Option<i16>,
    pub raw_eCO2: Option<i16>,
    pub tvoc: Option<i16>,
    pub State: Option<String>,
    pub Mode: Option<String>,
    pub l1: Option<bool>,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TelemetryPackDMT {
    pub timestamp: String,
    pub dev_id: String,
    pub samplingTime: i64,
    #[serde_as(as = "Vec<Option<BoolWrap>>")]
    pub Feedback: Vec<Option<bool>>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TelemetryDMT {
    pub timestamp: String,
    pub F1: Option<bool>,
    pub dev_id: String,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TelemetryPackDAL {
    pub timestamp: String,
    pub dev_id: String,
    pub State: String,
    pub Mode: Vec<String>,
    #[serde_as(as = "Vec<Option<BoolWrap>>")]
    pub Feedback: Vec<Option<bool>>,
    #[serde_as(as = "Vec<Option<BoolWrap>>")]
    pub Relays: Vec<Option<bool>>,
    pub gmt: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TelemetryDAL {
    pub timestamp: String,
    pub dev_id: String,
    pub State: String,
    pub Mode: Vec<String>,
    pub Feedback: Vec<Option<bool>>,
    pub Relays: Vec<Option<bool>>,
    pub gmt: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TelemetryRawDAM_v1 {
    pub timestamp: String,
    pub State: String,
    pub Mode: String,
    pub Temperature: Option<String>,
    pub Temperature_1: Option<String>,
    pub gmt: Option<String>,
}

fn f64_from_str<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    match Value::deserialize(deserializer) {
        Ok(Value::Number(result)) => result.as_f64().ok_or_else(|| {
            Error::invalid_type(Unexpected::Other(&result.to_string()), &"Tipo incorreto")
        }),

        Ok(Value::String(result)) => result
            .parse::<f64>()
            .map_err(|e| Error::invalid_value(Unexpected::Str(&result), &"Float em String")),
        Ok(wrong_value) => Err(Error::invalid_type(
            Unexpected::Other(&wrong_value.to_string()),
            &"Tipo não adequado",
        )),
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
    Temp2(Option<f64>),
}

impl SerializeAs<f64> for VerifyStringOrf64 {
    fn serialize_as<S>(source: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_f64(*source)
    }
}

impl<'de> DeserializeAs<'de, f64> for VerifyStringOrf64 {
    fn deserialize_as<D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        f64_from_str(deserializer)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct BoolWrap(bool);

impl SerializeAs<bool> for BoolWrap {
    fn serialize_as<S>(source: &bool, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bool(*source)
    }
}

impl<'de> DeserializeAs<'de, bool> for BoolWrap {
    fn deserialize_as<D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
    {
        bool_from_int(deserializer)
    }
}


fn bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match u8::deserialize(deserializer)? {
        0 => Ok(false),
        1 => Ok(true),
        other => Err(Error::invalid_value(
            Unexpected::Unsigned(other as u64),
            &"zero or one",
        )),
    }
}

#[derive(Debug, Serialize)]
pub struct TelemetryDUTv2<'a> {
    pub timestamp: NaiveDateTime,
    pub sampling_time: i64,
    pub temp: Option<f64>,
    pub temp_1: Option<f64>,
    pub hum: Option<f64>,
    pub e_co2: Option<i16>,
    pub tvoc: Option<i16>,
    pub state: Option<&'a str>,
    pub mode: Option<&'a str>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TelemetryDAC_v3 {
    pub timestamp: String,
    pub Lcmp: Option<bool>,
    pub Tamb: Option<f64>,
    pub Tsuc: Option<f64>,
    pub Tliq: Option<f64>,
    pub Psuc: Option<f64>,
    pub Pliq: Option<f64>,
    pub saved_data: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TelemetryDAC_v3_calcs {
    pub Tsh: Option<f64>,
    pub Tsc: Option<f64>,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct TelemetryPackDAC_v2 {
    pub timestamp: String,
    pub samplingTime: i64,
    #[serde_as(as = "Vec<Option<BoolWrap>>")]
    pub L1: Vec<Option<bool>>,
    pub T0: Vec<Option<f64>>,
    pub T1: Vec<Option<f64>>,
    pub T2: Vec<Option<f64>>,
    pub P0: Vec<Option<i16>>,
    pub P1: Vec<Option<i16>>,
    pub State: Option<String>,
    pub Mode: Option<String>,
    pub saved_data: Option<bool>,
}

pub struct TelemetryDACv2 {
    pub timestamp: NaiveDateTime,
    pub l1: Option<bool>,
    pub p0: Option<i16>,
    pub p1: Option<i16>,
}

