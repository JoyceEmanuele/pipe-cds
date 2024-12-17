use crate::schema::energy_monthly_consumption_target;
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};
use rust_decimal::Decimal;


#[derive(Debug, Queryable, Insertable, Clone)]
#[table_name = "energy_monthly_consumption_target"]
pub struct EnergyMonthlyConsumptionTarget {
    pub unit_id: i32,
    pub consumption_target: Decimal,
    pub date_forecast: NaiveDateTime
}
