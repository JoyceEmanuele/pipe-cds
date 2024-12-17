use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
pub struct UnitInfo {
    #[serde(rename = "UNIT_ID")]
    pub unit_id: i32,
    #[serde(rename = "UNIT_NAME")]
    pub unit_name: String,
    #[serde(rename = "CLIENT_NAME")]
    pub client_name: String,
    #[serde(rename = "CITY_NAME")]
    pub city_name: Option<String>,
    #[serde(rename = "STATE_NAME")]
    pub state_name: Option<String>,
    #[serde(rename = "TARIFA_KWH")]
    pub tarifa_kwh: Option<Decimal>,
    #[serde(rename = "CONSTRUCTED_AREA")]
    pub constructed_area: Option<Decimal>,
    #[serde(rename = "CAPACITY_POWER")]
    pub capacity_power: Option<Decimal>,
    #[serde(rename = "PRODUCTION_TIMESTAMP")]   
    pub production_timestamp: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UnitListData {
    pub list: Vec<UnitInfo>,
}
