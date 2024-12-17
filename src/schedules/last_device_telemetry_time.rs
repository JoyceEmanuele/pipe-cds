use std::{future::Future, pin::Pin, sync::Arc};

use chrono::{NaiveDateTime, NaiveDate};

use crate::{db::entities::last_device_telemetry_time::{get_last_telemetry_time, insert_last_device_telemetry_time}, models::{database_models::last_device_telemetry_time::LastDeviceTelemetryTime, external_models::device::DacDevice}, schedules::devices::{process_dac_common, process_single_dac_device}, GlobalVars};

pub fn insert_last_device_telemetry(
    record_date: &str,
    device_code: &str,
    globs: &Arc<GlobalVars>
) {
    let history = LastDeviceTelemetryTime {
        record_date: NaiveDateTime::parse_from_str(record_date, "%Y-%m-%dT%H:%M:%S").unwrap_or_default(),
        device_code: device_code.to_string(),
    };

    insert_last_device_telemetry_time(history, globs);
}

pub async fn very_last_device_telemetry_dac(
    day: &str,
    unit_id: i32,
    dac_device: &DacDevice,
    client_minutes_to_check_offline: Option<i32>,
    globs: &Arc<GlobalVars>
) -> Result<(), String> {

    let last_telemetry_time = match get_last_telemetry_time(dac_device.device_code.to_string(), globs) {
        Ok(Some(hist)) => hist,
        Ok(None) => return Ok(()),
        Err(err) => {
            eprintln!("Error finding last device telemetry, {:?}", err);
            return Err(format!("Error: {:?}", err));
        }
    };

    let day_parsed = NaiveDateTime::parse_from_str(&format!("{}T00:00:00", day), "%Y-%m-%dT%H:%M:%S").unwrap();

    if last_telemetry_time.record_date < day_parsed {
        let duration = day_parsed - last_telemetry_time.record_date;
        let hours_diff = duration.num_hours();

        if hours_diff > 1 {
            let mut current_day = last_telemetry_time.record_date.date();

            while current_day < day_parsed.date() {
                let day_to_process = current_day.format("%Y-%m-%d").to_string();
                process_dac_common(
                    unit_id,
                    &day_to_process,
                    dac_device,
                    client_minutes_to_check_offline,
                    globs,
                )
                .await?;
        
                current_day = current_day.succ_opt().unwrap();
            }
        }
    }

    Ok(())
}
