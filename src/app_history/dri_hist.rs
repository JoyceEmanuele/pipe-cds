use std::{collections::HashMap, error::Error, sync::Arc};

use chrono::{ NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{compression::compiler_DRI::{DRICCNCompiledPeriod, DRICCNTelemetryCompiler, DRIVAVandFancoilCompiledPeriod, DRIVAVandFancoilTelemetryCompiler}, db::config::dynamo::QuerierDevIdTimestamp, models::external_models::device::DriDevice, telemetry_payloads::dri_telemetry::{split_pack_ccn, split_pack_vav_and_fancoil, DriCCNTelemetry, DriChillerCarrierHXTelemetry, DriChillerCarrierXAHvarTelemetry, DriChillerCarrierXATelemetry, DriVAVandFancoilTelemetry, TelemetryDri, TelemetryDriChillerCarrierHX, TelemetryDriChillerCarrierXA, TelemetryDriChillerCarrierXAHvar}, GlobalVars};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DriHistParams {
    pub dev_id: String,
    pub dri_type: String,
    pub dri_interval: Option<isize>,
    pub day: NaiveDate,
    pub formulas: Option<HashMap<String, String>>,
    pub check_minutes_offline: Option<i32>,
}

impl DriHistParams {
    pub fn parse_parameters_dri(dri_device: &DriDevice, day: &str, check_minutes_offline: Option<i32>) -> Result<DriHistParams, Box<dyn Error>> {
        if dri_device.dri_type.is_none() {
            return Err("Missing DRI_TYPE".into())
        }

        Ok(DriHistParams {
            dev_id: dri_device.dev_id.clone(),
            dri_type: dri_device.dri_type.clone().unwrap(),
            dri_interval: dri_device.dri_interval,
            day: NaiveDate::parse_from_str(day, "%Y-%m-%d").unwrap_or_default(),
            formulas: dri_device.formulas.clone(),
            check_minutes_offline,
        })
    }

    pub async fn process_query(self, globs: &Arc<GlobalVars>) -> Result<String, Box<dyn Error>> {
        let tels = match &self.dri_type[..] {
            "CCN" => self.process_ccn_query(globs).await?.map(|result| result),
            "VAV" => self.process_vav_and_fancoil_query(globs).await?.map(|result| result),
            "FANCOIL" => self.process_vav_and_fancoil_query(globs).await?.map(|result| result),
            "CHILLER_CARRIER_HX" => self.process_chiller_carrier_query(globs).await?.map(|result| result),
            "CHILLER_CARRIER_XA" => self.process_chiller_carrier_query(globs).await?.map(|result| result),
            "CHILLER_CARRIER_XA_HVAR" => self.process_chiller_carrier_query(globs).await?.map(|result| result),
            _ => return Err("Unknown DRI type!".into()),
        };

        Ok(tels.unwrap_or_else(|| String::from("")))
    }

    async fn process_ccn_query(&self, globs: &Arc<GlobalVars>) -> Result<Option<String>, String> {
        let dev_id_upper = self.dev_id.to_uppercase();
        let mut table_name = {
            if (self.dev_id.len() == 12) && dev_id_upper.starts_with("DRI") {
                format!("{}XXXX_RAW", &dev_id_upper[0..8])
            } else {
                String::new()
            }
        };

        for custom in &globs.configfile.CUSTOM_TABLE_NAMES_DRI {
            if dev_id_upper.starts_with(&custom.dev_prefix) {
                table_name = custom.table_name.to_owned();
                break;
            }
        }

        if table_name.is_empty() {
            return Err(format!("Unknown DRI generation: {}", self.dev_id));
        }

        let interval_length_s = 24 * 60 * 60;
        let (ts_ini, ts_end) = {
            let i_ts_ini = self.day.and_hms(0, 0, 0).timestamp();
            
            let i_ts_end = i_ts_ini + interval_length_s;
            let ts_ini = NaiveDateTime::from_timestamp(i_ts_ini, 0).format("%Y-%m-%dT%H:%M:%S").to_string();
            let ts_end = NaiveDateTime::from_timestamp(i_ts_end, 0).format("%Y-%m-%dT%H:%M:%S").to_string();
            (ts_ini, ts_end)
        };

        let mut tcomp = DRICCNTelemetryCompiler::new(self.dri_interval);

        let querier = QuerierDevIdTimestamp::new_diel_dev(table_name, self.dev_id.to_owned());
        let mut final_tels = Vec::new();
        querier.run(&ts_ini, &ts_end, &mut |items: Vec<TelemetryDri>| {
            let mut x = items.into_iter()
                .filter_map(|tel| tel.try_into().ok())
                .collect::<Vec<DriCCNTelemetry>>();
            final_tels.append(&mut x);
            Ok(())
        }).await?;

        let day = self.day.to_string();
        let ts_ini = day.as_str();

        let i_ts_ini = match NaiveDateTime::parse_from_str(
            &format!("{}T00:00:00", ts_ini),
            "%Y-%m-%dT%H:%M:%S",
        ) {
            Err(err) => {
                println!("{} {}", &format!("{}T00:00:00", ts_ini), err);
                return Err(err.to_string());
            }
            Ok (mut date) => {
                date.timestamp()
            },
        };

        let i_ts_end = i_ts_ini + interval_length_s;
        let ts_end = NaiveDateTime::from_timestamp(i_ts_end + 10, 0)
            .format("%Y-%m-%dT%H:%M:%S")
            .to_string();

        for item in final_tels.iter() {
            let result = split_pack_ccn(
                item,
                i_ts_ini,
                i_ts_end,
                &mut |item: &DriCCNTelemetry, index: isize| {
                    tcomp.AdcPontos(item, index as isize);
                },
            );
            match result {
                Ok(()) => {}
                Err(err) => return Err(err),
            };
        }

        let period_data = tcomp.CheckClosePeriod(isize::try_from(interval_length_s).unwrap(), self.check_minutes_offline, &format!("{}T00:00:00", ts_ini),);
        let result = match period_data {
            Ok(v) => Some(v),
            Err(_) => None,
        };

        Ok(result.unwrap())
    }
    async fn process_vav_and_fancoil_query(&self, globs: &Arc<GlobalVars>) -> Result<Option<String>, String> {
        let dev_id_upper = self.dev_id.to_uppercase();
        let mut table_name = {
            if (self.dev_id.len() == 12) && dev_id_upper.starts_with("DRI") {
                format!("{}XXXX_RAW", &dev_id_upper[0..8])
            } else {
                String::new()
            }
        };

        for custom in &globs.configfile.CUSTOM_TABLE_NAMES_DRI {
            if dev_id_upper.starts_with(&custom.dev_prefix) {
                table_name = custom.table_name.to_owned();
                break;
            }
        }

        if table_name.is_empty() {
            return Err(format!("Unknown DRI generation: {}", self.dev_id));
        }

        let interval_length_s = 24 * 60 * 60;
        let (ts_ini, ts_end) = {
            let i_ts_ini = self.day.and_hms(0, 0, 0).timestamp();
            let i_ts_end = i_ts_ini + interval_length_s;
            let ts_ini = NaiveDateTime::from_timestamp(i_ts_ini, 0).format("%Y-%m-%dT%H:%M:%S").to_string();
            let ts_end = NaiveDateTime::from_timestamp(i_ts_end, 0).format("%Y-%m-%dT%H:%M:%S").to_string();
            (ts_ini, ts_end)
        };

        let mut tcomp = DRIVAVandFancoilTelemetryCompiler::new(self.dri_interval);

        let querier = QuerierDevIdTimestamp::new_diel_dev(table_name, self.dev_id.to_owned());
        let mut final_tels = Vec::new();
        querier.run(&ts_ini, &ts_end, &mut |items: Vec<TelemetryDri>| {
            let mut x = items.into_iter()
                .filter_map(|mut tel| {
                    tel.formulas = self.formulas.clone();
                    tel.try_into().ok()
                })
                .collect::<Vec<DriVAVandFancoilTelemetry>>();
            final_tels.append(&mut x);
            Ok(())
        }).await?;

        let day = self.day.to_string();
        let ts_ini = day.as_str();
        let i_ts_ini = match NaiveDateTime::parse_from_str(
            &format!("{}T00:00:00", ts_ini),
            "%Y-%m-%dT%H:%M:%S",
        ) {
            Err(err) => {
                println!("{} {}", &format!("{}T00:00:00", ts_ini), err);
                return Err(err.to_string());
            }
            Ok(mut date) => {
                date.timestamp()
              },
        };
        let i_ts_end = i_ts_ini + interval_length_s;
        let ts_end = NaiveDateTime::from_timestamp(i_ts_end + 10, 0)
            .format("%Y-%m-%dT%H:%M:%S")
            .to_string();

        for item in final_tels.iter() {
            let result = split_pack_vav_and_fancoil(
                item,
                i_ts_ini,
                i_ts_end,
                &mut |item: &DriVAVandFancoilTelemetry, index: isize| {
                    tcomp.AdcPontos(item, index as isize);
                },
            );
            match result {
                Ok(()) => {}
                Err(err) => return Err(err),
            };
        }

        let period_data = tcomp.CheckClosePeriod(isize::try_from(interval_length_s).unwrap(), self.check_minutes_offline, &format!("{}T00:00:00", ts_ini));
        let result = match period_data {
            Ok(v) => Some(v),
            Err(_) => None,
        };

        Ok(result.unwrap())
    }

    async fn process_chiller_carrier_query(&self, globs: &Arc<GlobalVars>) -> Result<Option<String>, String> {
        let dev_id_upper = self.dev_id.to_uppercase();
        let mut table_name = {
            if (self.dev_id.len() == 12) && dev_id_upper.starts_with("DRI") {
                format!("{}XXXX_RAW", &dev_id_upper[0..8])
            } else {
                String::new()
            }
        };

        for custom in &globs.configfile.CUSTOM_TABLE_NAMES_DRI {
            if dev_id_upper.starts_with(&custom.dev_prefix) {
                table_name = custom.table_name.to_owned();
                break;
            }
        }

        if table_name.is_empty() {
            return Err(format!("Unknown DRI generation: {}", self.dev_id));
        }

        let interval_length_s = 24 * 60 * 60;
        let (ts_ini, ts_end) = {
            let i_ts_ini = self.day.and_hms(0, 0, 0).timestamp();
            let i_ts_end = i_ts_ini + interval_length_s;
            let ts_ini = NaiveDateTime::from_timestamp(i_ts_ini, 0).format("%Y-%m-%dT%H:%M:%S").to_string();
            let ts_end = NaiveDateTime::from_timestamp(i_ts_end, 0).format("%Y-%m-%dT%H:%M:%S").to_string();
            (ts_ini, ts_end)
        };

        if &self.dri_type == "CHILLER_CARRIER_XA_HVAR" {
            let final_tels = self.process_chiller_carrier_xa_hvar_query(globs, ts_ini, ts_end, table_name).await?;
            Ok(Some(serde_json::to_string(&final_tels).unwrap()))
        } else if &self.dri_type == "CHILLER_CARRIER_XA" {
            let final_tels = self.process_chiller_carrier_xa_query(globs, ts_ini, ts_end, table_name).await?;
            Ok(Some(serde_json::to_string(&final_tels).unwrap()))
        }
        else {
            let final_tels = self.process_chiller_carrier_hx_query(globs, ts_ini, ts_end, table_name).await?;
            Ok(Some(serde_json::to_string(&final_tels).unwrap()))
        } 


    }

    async fn process_chiller_carrier_hx_query(&self, globs: &Arc<GlobalVars>, ts_ini: String, ts_end: String, table_name: String) -> Result<Vec<DriChillerCarrierHXTelemetry>, String> {
        let querier = QuerierDevIdTimestamp::new_diel_dev(table_name, self.dev_id.to_owned());
        let mut final_tels = Vec::new();
        querier.run(&ts_ini, &ts_end, &mut |items: Vec<TelemetryDriChillerCarrierHX>| {
            let mut x = items.into_iter()
                .filter_map(|mut tel| {
                    tel.formulas = self.formulas.clone();
                    tel.try_into().ok()
                })
                .collect::<Vec<DriChillerCarrierHXTelemetry>>();
            final_tels.append(&mut x);
            Ok(())
        }).await?;

        Ok(final_tels)
    }

    async fn process_chiller_carrier_xa_query(&self, globs: &Arc<GlobalVars>, ts_ini: String, ts_end: String, table_name: String) -> Result<Vec<DriChillerCarrierXATelemetry>, String> {
        let querier = QuerierDevIdTimestamp::new_diel_dev(table_name, self.dev_id.to_owned());
        let mut final_tels = Vec::new();
        querier.run(&ts_ini, &ts_end, &mut |items: Vec<TelemetryDriChillerCarrierXA>| {
            let mut x = items.into_iter()
                .filter_map(|mut tel| {
                    tel.formulas = self.formulas.clone();
                    tel.try_into().ok()
                })
                .collect::<Vec<DriChillerCarrierXATelemetry>>();
            final_tels.append(&mut x);
            Ok(())
        }).await?;

        Ok(final_tels)
    }

    async fn process_chiller_carrier_xa_hvar_query(&self, globs: &Arc<GlobalVars>, ts_ini: String, ts_end: String, table_name: String) -> Result<Vec<DriChillerCarrierXAHvarTelemetry>, String> {
        let querier = QuerierDevIdTimestamp::new_diel_dev(table_name, self.dev_id.to_owned());
        let mut final_tels = Vec::new();
        querier.run(&ts_ini, &ts_end, &mut |items: Vec<TelemetryDriChillerCarrierXAHvar>| {
            let mut x = items.into_iter()
                .filter_map(|mut tel| {
                    tel.formulas = self.formulas.clone();
                    tel.try_into().ok()
                })
                .collect::<Vec<DriChillerCarrierXAHvarTelemetry>>();
            final_tels.append(&mut x);
            Ok(())
        }).await?;

        Ok(final_tels)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DriCompiledPeriod {
    DRICCNCompiledPeriod(DRICCNCompiledPeriod),
    DRIVAVandFancoilCompiledPeriod(DRIVAVandFancoilCompiledPeriod),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DriHist {
    pub hours_online: Decimal
}
