use crate::schema::device_disponibility_hist;
use chrono::NaiveDate;
use diesel::{Insertable, Queryable};
use rust_decimal::Decimal;

#[derive(Debug, Queryable, Insertable)]
#[table_name = "device_disponibility_hist"]
pub struct DeviceDisponibilityHist {
    pub unit_id: i32,
    pub device_code: String,
    pub disponibility: Decimal,
    pub record_date: NaiveDate,
}
