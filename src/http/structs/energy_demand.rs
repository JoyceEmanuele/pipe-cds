use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use diesel::{sql_types::{Bool, Integer, Nullable, Numeric, Text, Timestamp, BigInt }, QueryableByName};

#[derive(Deserialize, Debug)]
pub struct ReqParamsGetDemandEnergy {
    pub start_date: String,
    pub end_date: String,
    pub unit_id: i32,
    pub hour_graphic: bool,
    pub electric_circuits_ids: Vec<i32>,
}

#[derive(QueryableByName, Deserialize, Serialize, Clone)]
pub struct GetEnergyDemandResponse {
    #[diesel(sql_type = Timestamp)]
    pub record_date: NaiveDateTime,
    #[diesel(sql_type = Numeric)]
    pub average_demand: Decimal,
    #[diesel(sql_type = Numeric)]
    pub max_demand: Decimal,
    #[diesel(sql_type = Numeric)]
    pub min_demand: Decimal,
}

#[derive(QueryableByName, Deserialize, Serialize, Clone, Debug)]
pub struct GetDemandInfoResponse {
    #[diesel(sql_type = Numeric)]
    pub average_demand: Decimal,
    #[diesel(sql_type = Numeric)]
    pub max_demand: Decimal,
    #[diesel(sql_type = Numeric)]
    pub min_demand: Decimal,
    #[diesel(sql_type = Timestamp)]
    pub max_demand_date: NaiveDateTime,
    #[diesel(sql_type = Timestamp)]
    pub min_demand_date: NaiveDateTime,
    #[diesel(sql_type = Numeric)]
    pub sum_demand: Decimal,
    #[diesel(sql_type = BigInt)]
    pub qtd_demand: i64,
}
