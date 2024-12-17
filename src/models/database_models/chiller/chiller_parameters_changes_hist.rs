use crate::schema::chiller_parameters_changes_hist;
use chrono::NaiveDateTime;
use serde::Serialize;
use diesel::{sql_types::{Integer, Numeric, Text, Timestamp}, Insertable, Queryable, QueryableByName};


#[derive(Debug, Queryable, Insertable, QueryableByName, Serialize)]
#[table_name = "chiller_parameters_changes_hist"]
pub struct ChillerParametersChangesHist {
    #[diesel(sql_type = Text)]
    pub device_code: String,
    #[diesel(sql_type = Integer)]
    pub unit_id: i32,
    #[diesel(sql_type = Text)]
    pub parameter_name: String,
    #[diesel(sql_type = Timestamp)]
    pub record_date: NaiveDateTime,
    #[diesel(sql_type = Integer)]
    pub parameter_value: i32,
}
