use std::sync::Arc;

use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::{db::entities::device_disponibility_hist::insert_data_device_disponibility_hist, models::database_models::device_disponibility_hist::DeviceDisponibilityHist, GlobalVars};


pub fn insert_device_disponibility_hist(
    unit_id: i32,
    disponibility: Decimal,
    day: &str,
    device_code: &str,
    globs: &Arc<GlobalVars>
) {
    let history = DeviceDisponibilityHist {
        unit_id,
        record_date: NaiveDate::parse_from_str(day, "%Y-%m-%d")
            .unwrap_or_default(),
        device_code: device_code.to_string(),
        disponibility: Decimal::from_str_exact(&format!("{:.4}", &disponibility)).unwrap().round_dp(2),
    };

    insert_data_device_disponibility_hist(history, globs);
}
