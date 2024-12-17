use crate::schema::devices_l1_totalization_hist;
use chrono::NaiveDate;
use serde::Serialize;
use diesel::{sql_types::{Integer, Numeric, Text, Date}, Insertable, Queryable, QueryableByName};
use rust_decimal::Decimal;


#[derive(Debug, Queryable, Insertable, QueryableByName, Serialize)]
#[table_name = "devices_l1_totalization_hist"]
pub struct DevicesL1TotalizationHist {
    #[diesel(sql_type = Integer)]
    pub asset_reference_id: Option<i32>,
    #[diesel(sql_type = Integer)]
    pub machine_reference_id: i32,
    #[diesel(sql_type = Text)]
    pub device_code: String,
    #[diesel(sql_type = Integer)]
    pub seconds_on: i32,
    #[diesel(sql_type = Integer)]
    pub seconds_off: i32,
    #[diesel(sql_type = Integer)]
    pub seconds_on_outside_programming: i32,
    #[diesel(sql_type = Integer)]
    pub seconds_must_be_off: i32,
    #[diesel(sql_type = Numeric)]
    pub percentage_on_outside_programming: Decimal,
    #[diesel(sql_type = Text)]
    pub programming: String,
    #[diesel(sql_type = Date)]
    pub record_date: NaiveDate,
}
