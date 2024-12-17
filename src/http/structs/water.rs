use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use diesel::{sql_types::{BigInt, Date, Integer, Nullable, Numeric, Text, Timestamp}, QueryableByName};
use chrono::{NaiveDate, NaiveDateTime};


#[derive(Deserialize, Clone)]
pub struct GetWaterUsageRequestBody {
    pub unitIds: Vec<i32>,
    pub startDate: String,
    pub endDate: String,
}
#[derive(Deserialize, Clone)]
pub struct GetWaterYearUsageRequestBody {
    pub unitIds: Vec<i32>,
    pub startDate: String,
    pub endDate: String,
}

#[derive(QueryableByName, Serialize)]
pub struct GetWaterUsageResponse {
  #[diesel(sql_type = Integer)]
  unit_id: i32,
  #[diesel(sql_type = Text)]
  device_code: String,
  #[diesel(sql_type = Numeric)]
  consumption: Decimal,
  #[diesel(sql_type = Timestamp)]
  compilation_record_date: NaiveDateTime,
}
#[derive(QueryableByName, Serialize)]
pub struct GetWaterYearUsageResponse {
  #[diesel(sql_type = Integer)]
  unit_id: i32,
  #[diesel(sql_type = Text)]
  device_code: String,
  #[diesel(sql_type = Numeric)]
  consumption: Decimal,
  #[diesel(sql_type = Timestamp)]
  compilation_record_date: NaiveDateTime,
}


#[derive(Deserialize, Clone)]
pub struct monthResquestType {
    pub year: i32,
    pub month: i32,
}


#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GetLastValidConsumption {
    pub consumption: Decimal,
    pub record_date: NaiveDateTime,
}

#[derive(Deserialize, Clone)]
pub struct GetWaterUsageHistoryRequest {
    pub unit_id: i32,
    pub start_date: String,
    pub end_date: String,
    pub last_start_date: String,
    pub last_end_date: String,
    pub hour_graphic: bool,
    pub year_graphic: bool,
}

#[derive(QueryableByName, Serialize, Debug)]
pub struct GetWaterUsageHistoryResponse {
  #[diesel(sql_type = Text)]
  pub device_code: String,
  #[diesel(sql_type = Numeric)]
  usage: Decimal,
  #[diesel(sql_type = Timestamp)]
  information_date: NaiveDateTime,
}

#[derive(QueryableByName, Deserialize, Serialize, Clone, Debug)]
pub struct GetWaterGraphicInfoResponse {
  #[diesel(sql_type = Numeric)]
  consumption: Decimal,
  #[diesel(sql_type = Numeric)]
  average_consumption: Decimal,
  #[diesel(sql_type = BigInt)]
  pub qtd_consumption: i64,
}


#[derive(QueryableByName, Serialize, Deserialize, Debug)]
pub struct GetWaterDayGraphicInfoResponse {
  #[diesel(sql_type = Numeric)]
  consumption: Decimal,
  #[diesel(sql_type = Numeric)]
  average_consumption: Decimal,
  #[diesel(sql_type = BigInt)]
  pub qtd_consumption: i64,
}

#[derive(QueryableByName, Serialize, Deserialize)]
pub struct GetWaterConsumption {
  #[diesel(sql_type = Numeric)]
  pub consumption: Decimal,
}

#[derive(Deserialize, Clone)]
pub struct GetWaterForecastUsageRequestBody {
    pub forecast_date: String,
    pub unit_id: i32,
}

#[derive(QueryableByName, Serialize, Deserialize)]
pub struct GetWaterForecastUsageResponse {
  #[diesel(sql_type = Numeric)]
  monday: Decimal,
  #[diesel(sql_type = Numeric)]
  tuesday: Decimal,
  #[diesel(sql_type = Numeric)]
  wednesday: Decimal,
  #[diesel(sql_type = Numeric)]
  thursday: Decimal,
  #[diesel(sql_type = Numeric)]
  friday: Decimal,
  #[diesel(sql_type = Numeric)]
  saturday: Decimal,
  #[diesel(sql_type = Numeric)]
  sunday: Decimal,
}
