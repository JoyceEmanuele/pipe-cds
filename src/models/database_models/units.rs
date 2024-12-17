use crate::schema::units;
use diesel::{sql_types::{Integer, Nullable, Numeric, Text}, Identifiable, Insertable, Queryable, QueryableByName};
use rust_decimal::Decimal;


#[derive(Debug, Queryable, Insertable, Identifiable, QueryableByName)]
#[table_name = "units"]
pub struct Units {
    #[diesel(sql_type = Nullable<Integer>)]
    pub id: Option<i32>,
    #[diesel(sql_type = Integer)]
    pub client_id: i32,
    #[diesel(sql_type = Text)]
    pub unit_name: String,
    #[diesel(sql_type = Integer)]
    pub reference_id: i32,
    #[diesel(sql_type = Nullable<Text>)]
    pub city_name: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub state_name: Option<String>,
    #[diesel(sql_type = Nullable<Numeric>)]
    pub tarifa_kwh: Option<Decimal>,
    #[diesel(sql_type = Nullable<Numeric>)]
    pub constructed_area: Option<Decimal>,
    #[diesel(sql_type = Nullable<Numeric>)]
    pub capacity_power: Option<Decimal>,
}
