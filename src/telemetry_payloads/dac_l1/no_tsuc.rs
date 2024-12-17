use crate::telemetry_payloads::{telemetry_formats::{TelemetryDAC_v3, TelemetryDACv2}, dac_telemetry::HwInfoDAC};

use super::dac_l1_calculator::DacL1Calculator;

#[derive(Debug, Default)]
pub(crate) struct NoTsucL1;

impl DacL1Calculator for NoTsucL1 {
    #[inline]
    fn calc_l1(
        &mut self,
        building_tel: &TelemetryDAC_v3,
        _tel: &TelemetryDACv2,
        _cfg: &HwInfoDAC,
    ) -> Result<Option<bool>, String> {
        let tamb= building_tel.Tamb;
        let tliq= building_tel.Tliq;

        Ok(
            tamb
                .zip(tliq)
                .map(|(ta, tl)| tl - ta >= 8.0)
        )
    }
}
