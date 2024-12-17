use crate::schema::energy_consumption_forecast;
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};
use rust_decimal::Decimal;


#[derive(Debug, Queryable, Insertable, Clone)]
#[table_name = "energy_consumption_forecast"]
pub struct EnergyConsumptionForecast {
    pub electric_circuit_id: i32,
    pub consumption_forecast: Decimal,
    pub record_date: NaiveDateTime
}
