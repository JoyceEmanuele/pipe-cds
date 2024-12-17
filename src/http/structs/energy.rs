use chrono::{NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use diesel::{sql_types::{BigInt, Bool, Integer, Nullable, Numeric, Text, Timestamp}, QueryableByName};

#[derive(Deserialize, Clone, Debug)]
pub enum OrderByTypeEnum {
    Asc,
    Desc,
}

#[derive(Deserialize, Clone, Debug)]
pub enum AnalysisHistTypeEnum {
    month,
    year,
}

#[derive(Deserialize, Clone, Debug)]
pub enum AnalysisHistFilterTypeEnum {
    CONSUMPTION,
    CONSUMPTION_FORECAST,
}

#[derive(Deserialize, Clone, Debug)]
pub enum categoryFilterEnum {
    A,
    B,
    C,
    D,
    E,
    F,
    G
}

#[derive(Deserialize, Clone, Debug)]
pub struct GetEnergyAnalysisListRequestBody {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub startDate: String,
    pub endDate: String,
    pub units: Vec<i32>,
    pub orderByField: Option<String>,
    pub orderByType: Option<OrderByTypeEnum>,
    pub isDielUser: bool,
    pub previousStartDate: String,
    pub previousEndDate: String,
    pub minConsumption: i32,
    pub categoryFilter: Option<Vec<categoryFilterEnum>>
}

#[derive(QueryableByName, Serialize, Debug)]
pub struct GetEnergyAnalysisListResponseSQL {
    #[diesel(sql_type = Text)]
    pub client_name: String,

    #[diesel(sql_type = Integer)]
    pub reference_id: i32,

    #[diesel(sql_type = Text)]
    pub unit_name: String,

    #[diesel(sql_type = Numeric)]
    pub consumption: Decimal,

    #[diesel(sql_type = Nullable<Numeric>)]
    pub refrigeration_consumption: Option<Decimal>,

    #[diesel(sql_type = Nullable<Numeric>)]
    pub capacity_power: Option<Decimal>,

    #[diesel(sql_type = Nullable<Numeric>)]
    pub refrigeration_consumption_percentage: Option<Decimal>,

    #[diesel(sql_type = Nullable<Numeric>)]
    pub consumption_by_area: Option<Decimal>,

    #[diesel(sql_type = Nullable<Numeric>)]
    pub refrigeration_consumption_by_area: Option<Decimal>,

    #[diesel(sql_type = Nullable<Text>)]
    pub city_name: Option<String>,

    #[diesel(sql_type = Nullable<Text>)]
    pub state_name: Option<String>,

    #[diesel(sql_type = Nullable<Numeric>)]
    pub total_charged: Option<Decimal>,

    #[diesel(sql_type = Numeric)]
    pub invalid_count: Decimal,
    
    #[diesel(sql_type = Numeric)]
    pub processed_count: Decimal,

    #[diesel(sql_type = Numeric)]
    pub readings_count: Decimal,

    #[diesel(sql_type = Nullable<BigInt>)]
    pub ranking: Option<i64>,

    #[diesel(sql_type = Nullable<Text>)]
    pub category: Option<String>,

    #[diesel(sql_type = Nullable<Numeric>)]
    pub previous_consumption: Option<Decimal>
}

#[derive(Serialize, Debug)]
pub struct GetEnergyAnalysisListResponseWithFlags {
    #[serde(rename = "clientName")]
    pub client_name: String,

    #[serde(rename = "unitId")]
    pub reference_id: i32,

    #[serde(rename = "unitName")]
    pub unit_name: String,
    
    pub consumption: Decimal,

    #[serde(rename = "refrigerationConsumption")]
    pub refrigeration_consumption: Option<Decimal>,

    #[serde(rename = "refCapacity")]
    pub capacity_power: Option<Decimal>,

    #[serde(rename = "refrigerationConsumptionPercentage")]
    pub refrigeration_consumption_percentage: Option<Decimal>,

    #[serde(rename = "consumptionByArea")]
    pub consumption_by_area: Option<Decimal>,

    #[serde(rename = "refrigerationConsumptionByArea")]
    pub refrigeration_consumption_by_area: Option<Decimal>,

    #[serde(rename = "cityName")]
    pub city_name: Option<String>,

    #[serde(rename = "stateName")]
    pub state_name: Option<String>,

    #[serde(rename = "totalCharged")]
    pub total_charged: Option<Decimal>,

    #[serde(rename = "dataIsInvalid")]
    pub invalid: bool,
    
    #[serde(rename = "dataIsProcessed")]
    pub processed: bool,

    #[serde(rename = "procelRanking")]
    pub ranking: Option<i64>,

    #[serde(rename = "procelCategory")]
    pub category: Option<String>,

    #[serde(rename = "consumptionPreviousPercentage")]
    pub previous_consumption: Option<Decimal>
}

#[derive(Serialize, Debug)]
pub struct GetEnergyAnalysisListResponse {
    pub units: Vec<GetEnergyAnalysisListResponseSQL>,

    pub classA: i32,
    pub classB: i32,
    pub classC: i32,
    pub classD: i32,
    pub classE: i32,
    pub classF: i32,
    pub classG: i32,
}

#[derive(Serialize, Debug)]
pub struct GetEnergyAnalysisListResponseComplete {
    pub units: Vec<GetEnergyAnalysisListResponseWithFlags>,

    pub classA: i32,
    pub classB: i32,
    pub classC: i32,
    pub classD: i32,
    pub classE: i32,
    pub classF: i32,
    pub classG: i32,
}

#[derive(Deserialize, Clone)]
pub struct GetEnergyAnalysisHistRequestBody {
    pub units: Vec<i32>,
    pub startDate: String,
    pub endDate: String,
    pub filterType: AnalysisHistTypeEnum,
    pub isDielUser: bool,
    pub minConsumption: i32
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetEnergyAnalysisHistFilterRequestBody {
    pub units: Vec<i32>,
    pub date: String,
    pub filterType: AnalysisHistFilterTypeEnum,
}

#[derive(QueryableByName, Serialize)]
pub struct GetEnergyAnalysisHistResponse {
    #[diesel(sql_type = Numeric)]
    pub consumption: Decimal,

    #[diesel(sql_type = Timestamp)]
    pub time: chrono::NaiveDateTime,

    #[diesel(sql_type = Nullable<Numeric>)]
    pub total_charged: Option<Decimal>,

    #[diesel(sql_type = Numeric)]
    pub invalid_count: Decimal,

    #[diesel(sql_type = Numeric)]
    pub processed_count: Decimal,

    #[diesel(sql_type = Numeric)]
    pub readings_count: Decimal,

    #[diesel(sql_type = BigInt)]
    pub units_count: i64
}

#[derive(QueryableByName, Serialize)]
pub struct GetTotalUnitsWithConsumption {
    #[diesel(sql_type = BigInt)]
    pub units_count: i64
}

#[derive(Serialize, Debug)]
pub struct GetEnergyAnalysisHistResponseWithFlags {
    pub consumption: Decimal,

    pub time: chrono::NaiveDateTime,

    #[serde(rename = "totalCharged")]
    pub total_charged: Option<Decimal>,

    #[serde(rename = "dataIsInvalid")]
    pub invalid: bool,
    
    #[serde(rename = "dataIsProcessed")]
    pub processed: bool,

    #[serde(rename = "totalUnits")]
    pub units_count: i64
}

#[derive(QueryableByName, Serialize)]
pub struct GetEnergyAnalysisHistFilterResponse {
    #[diesel(sql_type = Timestamp)]
    time: chrono::NaiveDateTime
}

#[derive(Deserialize, Clone, Debug)]
pub struct GetUnitListRequestBody {
    pub units: Vec<i32>,
    pub startDate: String,
    pub endDate: String

}

#[derive(Deserialize, Clone, Debug)]
pub struct GetUnitListProcelRequestBody {
    pub units: Vec<i32>,
    pub startDate: String,
    pub endDate: String,
    pub minConsumption: i32

}

#[derive(QueryableByName, Serialize, Debug)]
pub struct GetUnitListResponse {
    #[diesel(sql_type = Integer)]
    pub unit_id: i32,

    #[diesel(sql_type = Nullable<Text>)]
    pub city_name: Option<String>,

    #[diesel(sql_type = Nullable<Text>)]
    pub state_name: Option<String>
}

#[derive(Deserialize, Clone)]
pub struct ReqParamsGetEnergyConsumption {
    pub start_date: String,
    pub end_date: String,
    pub unit_id: i32,
    pub get_hour_consumption: Option<bool>,
    pub isDielUser: bool
}

#[derive(QueryableByName, Deserialize, Serialize, Clone)]
pub struct GetDayEnergyConsumptionResponse {
    #[diesel(sql_type = Text)]
    pub day: String,
    #[diesel(sql_type = Numeric)]
    pub total_measured: Decimal,
    #[diesel(sql_type = Numeric)]
    pub max_day_total_measured: Decimal,
    #[diesel(sql_type = Integer)]
    pub electric_circuit_reference_id: i32,
    #[diesel(sql_type = BigInt)]
    pub invalid_count: i64,
    #[diesel(sql_type = BigInt)]
    pub processed_count: i64,
    #[diesel(sql_type = BigInt)]
    pub readings_count: i64
}

#[derive(QueryableByName, Deserialize, Serialize, Clone)]
pub struct GetHourEnergyConsumptionResponse {
    #[diesel(sql_type = Timestamp)]
    pub hour: NaiveDateTime,
    #[diesel(sql_type = Numeric)]
    pub total_measured: Decimal,
    #[diesel(sql_type = Integer)]
    pub electric_circuit_reference_id: i32,
    #[diesel(sql_type = Bool)]
    pub contains_invalid: bool,
    #[diesel(sql_type = Bool)]
    pub contains_processed: bool
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GetEnergyConsumptionResponse {
    pub energy_day_consumption_list: Vec<GetDayEnergyConsumptionResponse>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GetLastValidConsumption {
    pub consumption: Decimal,
    pub record_date: NaiveDateTime,
}

#[derive(QueryableByName, Serialize, Debug, Clone, Copy)]
pub struct GetUnitEnergyStats {
    #[diesel(sql_type = Numeric)]
    pub avg_consumption_by_area: Decimal,

    #[diesel(sql_type = Numeric)]
    pub max_consumption_by_area: Decimal,

    #[diesel(sql_type = Numeric)]
    pub min_consumption_by_area: Decimal
}

#[derive(QueryableByName, Serialize, Debug, Copy, Clone)]
pub struct GetUnitConsumptionByArea {
    #[diesel(sql_type = Integer)]
    pub unit_id: i32,

    #[diesel(sql_type = Numeric)]
    pub consumption_by_area: Decimal
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ProcelType {
    pub units: Vec<i32>,
    pub percentage: f64
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GetProcelInsigthsResponse {
    pub averageConsumption: Decimal,
    pub averageConsumptionPreviousMonthPercentage: Decimal,
    pub totalConsumption: Decimal,
    pub totalCharged: Decimal,
    pub containsProcel: bool,
    pub containsAnalysisData: bool,

    #[serde(rename = "a")]
    pub classA: ProcelType,

    #[serde(rename = "b")]
    pub classB: ProcelType,

    #[serde(rename = "c")]
    pub classC: ProcelType,

    #[serde(rename = "d")]
    pub classD: ProcelType,

    #[serde(rename = "e")]
    pub classE: ProcelType,

    #[serde(rename = "f")]
    pub classF: ProcelType,

    #[serde(rename = "g")]
    pub classG: ProcelType,
}

#[derive(QueryableByName, Serialize, Debug, Copy, Clone)]
pub struct GetGeneralUnitsStats {
    #[diesel(sql_type = Nullable<Numeric>)]
    pub total_consumption: Option<Decimal>,

    #[diesel(sql_type = Nullable<Numeric>)]
    pub total_charged: Option<Decimal>
}

#[derive(Deserialize, Clone)]
pub struct GetProcelInsightsRequestBody {
    pub units: Vec<i32>,
    pub startDate: String,
    pub endDate: String,
    pub previousStartDate: String,
    pub previousEndDate: String,
    pub minConsumption: i32,
    pub procelUnitsFilter: Option<Vec<i32>>
}

#[derive(Deserialize, Clone, Debug)]
pub struct GetEnergyTrendsRequestBody {
    pub units: Vec<i32>,
    pub startDate: String,
    pub endDate: String,
    pub days: i64

}

#[derive(QueryableByName, Serialize, Debug)]
pub struct GetTrendsSQL {
    #[diesel(sql_type = Nullable<Numeric>)]
    pub consumption: Option<Decimal>,

    #[diesel(sql_type = Numeric)]
    pub forecast: Decimal,

    #[diesel(sql_type = Timestamp)]
    pub time: chrono::NaiveDateTime,
}

#[derive(QueryableByName, Serialize, Debug)]
pub struct GetMonthlyTargetSQL {
    #[diesel(sql_type = Nullable<Numeric>)]
    pub target: Option<Decimal>
}

#[derive(Deserialize, Serialize, Clone)]
pub struct EnergyTrends {
    pub time: chrono::NaiveDateTime,
    pub consumption: Decimal,
    pub consumptionForecast: Decimal,
    pub consumptionTarget: Option<Decimal>,
    pub consumptionOverTarget: Decimal,
    pub consumptionPercentage: Decimal
}

#[derive(Deserialize, Serialize, Clone)]
pub struct GetEnergyTrendsResponse {

    pub trendsData: Vec<EnergyTrends>,
    pub monthlyForecast: Option<Decimal>,
    pub monthlyTarget: Option<Decimal>,
    pub totalConsumption: Option<Decimal>,
    pub monthlyForecastPercentage: Option<Decimal>,
    pub totalConsumtionPercentage: Option<Decimal>
}

#[derive(Deserialize, Clone, Debug)]
pub struct ParamsGetTotalDaysConsumptionUnit {
    pub unit_id: i32,
    pub start_date: String,
    pub end_date: String,
}

#[derive(QueryableByName, Serialize, Debug)]
pub struct GetTotalDaysConsumptionUnit {
    #[diesel(sql_type = BigInt)]
    pub days_count: i64
}

#[derive(QueryableByName, Serialize, Debug)]
pub struct GetTotalMonthlyTarget {
    #[diesel(sql_type = BigInt)]
    pub monthly_target_count: i64
}
