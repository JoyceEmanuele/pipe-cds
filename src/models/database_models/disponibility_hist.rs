use crate::schema::disponibility_hist;
use chrono::NaiveDate;
use diesel::{Insertable, Queryable};
use rust_decimal::Decimal;

#[derive(Debug, Queryable, Insertable)]
#[table_name = "disponibility_hist"]
pub struct DisponibilityHist {
    pub unit_id: i32,
    pub disponibility: Decimal,
    pub record_date: NaiveDate,

}