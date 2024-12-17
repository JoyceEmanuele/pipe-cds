use crate::schema::energy_demand_minutes_hist;
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};
use rust_decimal::Decimal;


#[derive(Debug, Queryable, Insertable, Clone)]
#[table_name = "energy_demand_minutes_hist"]
pub struct EnergyDemandMinutesHist {
    pub average_demand: Decimal,
    pub electric_circuit_id: i32,
    pub min_demand: Decimal,
    pub max_demand: Decimal,
    pub record_date: NaiveDateTime,
}
