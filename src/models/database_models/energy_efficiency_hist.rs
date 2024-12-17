use crate::schema::energy_efficiency_hist;
use chrono::NaiveDate;
use diesel::{Insertable, Queryable};
use rust_decimal::Decimal;

#[derive(Debug, Queryable, Insertable)]
#[table_name = "energy_efficiency_hist"]
pub struct EnergyEfficiencyHist {
    pub machine_id: i32,
    pub device_code: String,
    pub capacity_power: Decimal,
    pub consumption: Decimal,
    pub utilization_time: Decimal,
    pub record_date: NaiveDate,
}
