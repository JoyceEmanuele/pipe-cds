use crate::schema::water_consumption_forecast;
use chrono::NaiveDate;
use diesel::{Insertable, Queryable};
use rust_decimal::Decimal;

#[derive(Debug, Queryable, Insertable, Clone)]
#[table_name = "water_consumption_forecast"]
pub struct WaterConsumptionForecast {
    pub unit_id: i32,
    pub forecast_date: NaiveDate,
    pub monday: Option<Decimal>,
    pub tuesday: Option<Decimal>,
    pub wednesday: Option<Decimal>,
    pub thursday: Option<Decimal>,
    pub friday: Option<Decimal>,
    pub saturday: Option<Decimal>,
    pub sunday: Option<Decimal>,
}
