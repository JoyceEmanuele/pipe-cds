use crate::schema::water_hist;
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};
use rust_decimal::Decimal;

#[derive(Debug, Queryable, Insertable, Clone)]
#[table_name = "water_hist"]
pub struct WaterHist {
    pub unit_id: i32,
    pub supplier: String,
    pub record_date: NaiveDateTime,
    pub consumption: Decimal,
    pub device_code: String,
    pub is_measured_consumption: bool,
    pub is_valid_consumption: bool
}