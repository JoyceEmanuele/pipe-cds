use std::collections::{hash_map::RandomState, HashMap};

use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MachineAutomInterval {
    pub seconds_start: i32,
    pub seconds_end: i32,
    pub must_be_on: bool,
}

#[derive(Debug, Deserialize)]
pub struct Devices {
    pub dacs_devices: Option<Vec<DacDevice>>,
    pub duts_devices: Option<Vec<DutDevice>>,
    pub dma_device: Option<DmaDevice>,
    pub laager_device: Option<LaagerDevice>,
    pub energy_devices: Option<Vec<EnergyDevice>>,
    pub duts_to_disponibility: Option<Vec<DutDevice>>,
    pub dacs_to_disponibility: Option<Vec<DacDevice>>,
    pub dris_to_disponibility: Option<Vec<DriDevice>>,
    pub dmts_to_disponibility: Option<Vec<DmtDevice>>,
    pub dals_to_disponibility: Option<Vec<DalDevice>>,
    pub dams_to_disponibility: Option<Vec<DamDevice>>,
    pub dacs_to_l1_automation: Option<Vec<DacDevice>>,
    pub duts_to_l1_automation: Option<Vec<DutDevice>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DacDevice {
    #[serde(rename = "DEVICE_CODE")]
    pub device_code: String,
    #[serde(rename = "MACHINE_ID")]
    pub machine_id: Option<i32>,
    #[serde(rename = "MACHINE_NAME")]
    pub machine_name: Option<String>,
    #[serde(rename = "MACHINE_KW")]
    pub machine_kw: Option<Decimal>,
    #[serde(rename = "IS_VRF")]
    pub is_vrf: bool,
    #[serde(rename = "CALCULATE_L1_FANCOIL")]
    pub calculate_l1_fancoil: bool,
    #[serde(rename = "HAS_AUTOMATION")]
    pub has_automation: bool,
    #[serde(rename = "FLUID_TYPE")]
    pub fluid_type: Option<String>,
    #[serde(rename = "P0_PSUC")]
    pub p0_psuc: bool,
    #[serde(rename = "P1_PSUC")]
    pub p1_psuc: bool,
    #[serde(rename = "P0_PLIQ")]
    pub p0_pliq: bool,
    #[serde(rename = "P1_PLIQ")]
    pub p1_pliq: bool,
    #[serde(rename = "P0_MULT")]
    pub p0_mult: Option<f64>,
    #[serde(rename = "P1_MULT")]
    pub p1_mult: Option<f64>,
    #[serde(rename = "P0_OFST")]
    pub p0_ofst: Option<f64>,
    #[serde(rename = "P1_OFST")]
    pub p1_ofst: Option<f64>,
    #[serde(rename = "T0_T1_T2")]
    pub t0_t1_t2: Option<Vec<String>>,
    #[serde(rename = "VIRTUAL_L1")]
    pub virtual_l1: bool,
    #[serde(rename = "DEVICE_CODE_AUTOM")]
    pub device_code_autom: Option<String>,
    #[serde(rename = "ASSET_ID")]
    pub asset_id: Option<i32>,
    #[serde(rename = "ASSET_NAME")]
    pub asset_name: Option<String>,
    pub machine_autom_intervals: Option<Vec<MachineAutomInterval>>,
    #[serde(rename = "V_MAJOR")]
    pub v_major: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct DutDevice {
    #[serde(rename = "DEVICE_CODE")]
    pub device_code: String,
    #[serde(rename = "MACHINE_ID")]
    pub machine_id: Option<i32>,
    #[serde(rename = "MACHINE_NAME")]
    pub machine_name: Option<String>,
    #[serde(rename = "MACHINE_KW")]
    pub machine_kw: Option<Decimal>,
    #[serde(rename = "TEMPERATURE_OFFSET")]
    pub temperature_offset: Option<f64>,
    #[serde(rename = "DEVICE_CODE_AUTOM")]
    pub device_code_autom: Option<String>,
    #[serde(rename = "ASSET_ID")]
    pub asset_id: Option<i32>,
    #[serde(rename = "ASSET_NAME")]
    pub asset_name: Option<String>,
    pub machine_autom_intervals: Option<Vec<MachineAutomInterval>>,
    pub has_energy_efficiency: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct DmaDevice {
    #[serde(rename = "DEVICE_CODE")]
    pub device_code: String,
    #[serde(rename = "LITERS_PER_PULSE")]
    pub liters_per_pulse: Option<i32>,
    #[serde(rename = "INSTALLATION_DATE")]
    pub installation_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LaagerDevice {
    #[serde(rename = "LAAGER_CODE")]
    pub laager_code: String,
    #[serde(rename = "INSTALLATION_DATE")]
    pub installation_date: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EnergyDevice {
    #[serde(rename = "DEVICE_CODE")]
    pub device_code: String,
    #[serde(rename = "SERIAL")]
    pub serial: Option<String>,
    #[serde(rename = "MANUFACTURER")]
    pub manufacturer: String,
    #[serde(rename = "MODEL")]
    pub model: Option<String>,
    #[serde(rename = "ELECTRIC_CIRCUIT_ID")]
    pub electric_circuit_id: i32,
    #[serde(rename = "ELECTRIC_CIRCUIT_NAME")]
    pub electric_circuit_name: String,
    pub formulas: Option<HashMap<String, String, RandomState>>,
    #[serde(rename = "DRI_INTERVAL")]
    pub dri_interval: i32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DriDevice {
    #[serde(rename = "DEVICE_CODE")]
    pub dev_id: String,
    #[serde(rename = "DRI_TYPE")]
    pub dri_type: Option<String>,
    #[serde(rename = "DRI_INTERVAL")]
    pub dri_interval: Option<isize>,
    pub formulas: Option<HashMap<String, String, RandomState>>
}

#[derive(Debug, Deserialize)]
pub struct DmtDevice {
    #[serde(rename = "DEVICE_CODE")]
    pub device_code: String,
}

#[derive(Debug, Deserialize)]
pub struct DalDevice {
    #[serde(rename = "DEVICE_CODE")]
    pub device_code: String,
}

#[derive(Debug, Deserialize)]
pub struct DamDevice {
    #[serde(rename = "DEVICE_CODE")]
    pub device_code: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfigDevices {
    pub devices: Devices,
}


#[derive(Debug, Serialize)]
pub struct LaagerLoginRequestBody {
    pub grant_type: String,
    pub client_id: String,
    pub client_secret: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LaagerLoginResponseData {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u32,
    pub created_at: u32,
}

#[derive(Debug, Deserialize)]
pub struct VerifyLaagerData {
    pub rf_device_id: String,
    pub customer_id: String,
}

#[derive(Debug, Deserialize)]
pub struct WaterConsumption {
    pub history: Vec<WaterConsumptionHistory>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WaterConsumptionHistory {
    pub date: String,
    pub usage: f64,
    pub reading: Option<f64>
}