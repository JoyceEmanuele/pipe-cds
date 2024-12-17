use crate::schema::{waters_hist, water_hist};
use chrono::{NaiveDate, NaiveDateTime};
use diesel::{Insertable, Queryable};
use rust_decimal::Decimal;


#[derive(Debug, Queryable, Insertable)]
#[table_name = "waters_hist"]
pub struct WatersHist {
    pub unit_id: i32,
    pub supplier: String,
    pub record_date: NaiveDate,
    pub consumption: Decimal,
    pub device_code: String,
}