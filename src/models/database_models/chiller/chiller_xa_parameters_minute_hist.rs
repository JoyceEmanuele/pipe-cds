use crate::schema::chiller_xa_parameters_minutes_hist;
use chrono::NaiveDateTime;
use serde::Serialize;

use diesel::{sql_types::{Integer, Numeric, Text, Timestamp}, Insertable, Queryable, QueryableByName};

use rust_decimal::Decimal;

#[derive(Debug, Queryable, QueryableByName, Insertable, Serialize)]
#[table_name = "chiller_xa_parameters_minutes_hist"]
pub struct ChillerXAParametersMinutesHist {
    pub device_code: String,
    pub unit_id: i32,
    pub record_date: NaiveDateTime,
    pub cap_t: Decimal,
    pub cond_ewt: Decimal,
    pub cond_lwt: Decimal,
    pub cool_ewt: Decimal,
    pub cool_lwt: Decimal,
    pub ctrl_pnt: Decimal,
    pub dp_a: Decimal,
    pub dp_b: Decimal,
    pub hr_cp_a: Decimal,
    pub hr_cp_b: Decimal,
    pub hr_mach: Decimal,
    pub hr_mach_b: Decimal,
    pub oat: Decimal,
    pub op_a: Decimal,
    pub op_b: Decimal,
    pub sct_a: Decimal,
    pub sct_b: Decimal,
    pub slt_a: Decimal,
    pub slt_b: Decimal,
    pub sp: Decimal,
    pub sp_a: Decimal,
    pub sp_b: Decimal,
    pub sst_a: Decimal,
    pub sst_b: Decimal,
}

#[derive(Debug, QueryableByName, Serialize)]
pub struct ChillerXAParametersHistRow {
    #[diesel(sql_type = Text)]
    pub device_code: String,
    #[diesel(sql_type = Integer)]
    pub unit_id: i32,
    #[diesel(sql_type = Timestamp)]
    pub record_date: NaiveDateTime,
    #[diesel(sql_type = Numeric)]
    pub cap_t: Decimal,
    #[diesel(sql_type = Numeric)]
    pub cond_ewt: Decimal,
    #[diesel(sql_type = Numeric)]
    pub cond_lwt: Decimal,
    #[diesel(sql_type = Numeric)]
    pub cool_ewt: Decimal,
    #[diesel(sql_type = Numeric)]
    pub cool_lwt: Decimal,
    #[diesel(sql_type = Numeric)]
    pub ctrl_pnt: Decimal,
    #[diesel(sql_type = Numeric)]
    pub dp_a: Decimal,
    #[diesel(sql_type = Numeric)]
    pub dp_b: Decimal,
    #[diesel(sql_type = Numeric)]
    pub hr_cp_a: Decimal,
    #[diesel(sql_type = Numeric)]
    pub hr_cp_b: Decimal,
    #[diesel(sql_type = Numeric)]
    pub hr_mach: Decimal,
    #[diesel(sql_type = Numeric)]
    pub hr_mach_b: Decimal,
    #[diesel(sql_type = Numeric)]
    pub oat: Decimal,
    #[diesel(sql_type = Numeric)]
    pub op_a: Decimal,
    #[diesel(sql_type = Numeric)]
    pub op_b: Decimal,
    #[diesel(sql_type = Numeric)]
    pub sct_a: Decimal,
    #[diesel(sql_type = Numeric)]
    pub sct_b: Decimal,
    #[diesel(sql_type = Numeric)]
    pub slt_a: Decimal,
    #[diesel(sql_type = Numeric)]
    pub slt_b: Decimal,
    #[diesel(sql_type = Numeric)]
    pub sp: Decimal,
    #[diesel(sql_type = Numeric)]
    pub sp_a: Decimal,
    #[diesel(sql_type = Numeric)]
    pub sp_b: Decimal,
    #[diesel(sql_type = Numeric)]
    pub sst_a: Decimal,
    #[diesel(sql_type = Numeric)]
    pub sst_b: Decimal,
}
