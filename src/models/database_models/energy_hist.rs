use crate::schema::energy_hist;
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};
use rust_decimal::Decimal;


#[derive(Debug, Queryable, Insertable, Clone)]
#[table_name = "energy_hist"]
pub struct EnergyHist {
    pub electric_circuit_id: i32,
    pub consumption: Decimal,
    pub record_date: NaiveDateTime,
    pub is_measured_consumption: bool,
    pub is_valid_consumption: bool
}
