use crate::telemetry_payloads::{telemetry_formats::{TelemetryDAC_v3, TelemetryDACv2}, dac_telemetry::HwInfoDAC};

use super::dac_l1_calculator::DacL1Calculator;

#[derive(Debug, Default)]
pub(crate) struct DacL1Fancoil;

impl DacL1Calculator for DacL1Fancoil {
    fn calc_l1(
            &mut self,
            building_tel: &TelemetryDAC_v3,
            _full_tel: &TelemetryDACv2,
            _cfg: &HwInfoDAC,
        ) -> Result<Option<bool>, String> {
        let tsuc = building_tel.Tsuc;
        let tliq = building_tel.Tliq;
        Ok(if let (Some(tsuc), Some(tliq)) = (tsuc, tliq) {
            Some((tsuc - tliq) >= 1.5)
        } else {
            None
        })
    }
}
