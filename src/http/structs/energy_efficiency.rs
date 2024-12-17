use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use diesel::{sql_types::{Integer, Text, Numeric}, QueryableByName};

#[derive(Deserialize)]
pub struct ReqParamsGetTotalConsumptionByUnit {
    pub start_date: String,
    pub end_date: String,
    pub unit_id: i32,
}

#[derive(QueryableByName, Serialize, Default)]
pub struct GetTotalConsumptionByUnitResponse {
    #[diesel(sql_type = Numeric)]
    pub total_refrigeration_consumption: Decimal,
}

#[derive(QueryableByName, Serialize, Default)]
pub struct GetTotalConsumptionByDeviceMachineUnitResponse {
    #[diesel(sql_type = Integer)]
    pub machine_id: i32,
    #[diesel(sql_type = Text)]
    pub device_code: String,
    #[diesel(sql_type = Numeric)]
    pub total_refrigeration_consumption: Decimal,
    #[diesel(sql_type = Numeric)]
    pub total_utilization_time: Decimal,
}
