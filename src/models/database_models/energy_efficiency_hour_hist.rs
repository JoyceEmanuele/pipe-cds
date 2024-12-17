use crate::schema::energy_efficiency_hour_hist;
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};
use rust_decimal::Decimal;

#[derive(Debug, Queryable, Insertable)]
#[table_name = "energy_efficiency_hour_hist"]
pub struct EnergyEfficiencyHourHist {
    pub machine_id: i32,
    pub device_code: String,
    pub consumption: Decimal,
    pub utilization_time: Decimal,
    pub record_date: NaiveDateTime,
}
