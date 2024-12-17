use crate::schema::last_device_telemetry_time;
use chrono::NaiveDateTime;
use diesel::{sql_types::Timestamp, Insertable, Queryable, QueryableByName};
use serde::Serialize;

#[derive(Debug, Queryable, Insertable, Clone)]
#[table_name = "last_device_telemetry_time"]
pub struct LastDeviceTelemetryTime {
    pub device_code: String,
    pub record_date: NaiveDateTime,
}

#[derive(Debug, QueryableByName, Serialize)]
pub struct GetLastTelemetryTime {
    #[diesel(sql_type = Timestamp)]
    pub record_date: NaiveDateTime,
}
