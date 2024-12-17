use crate::telemetry_payloads::{dut_telemetry::HwInfoDUT, telemetry_formats::{TelemetryDUT_v3, TelemetryDUTv2}};

use super::temp_difference::TempDiffL1;

pub trait DutL1Calculator: Send + Sync {
    fn calc_l1(
        &mut self,
        payload: &TelemetryDUTv2,
        cfg: &HwInfoDUT,
    ) -> Result<Option<bool>, String>;

    fn calc_l1_tel_v3(
        &mut self,
        payload: &TelemetryDUT_v3,
        sampling_time: i64,
        cfg: &HwInfoDUT,
    ) -> Result<Option<bool>, String> {
        let p = TelemetryDUTv2 {
            temp: payload.Temp,
            temp_1: payload.Temp1,
            hum: payload.Hum,
            e_co2: payload.eCO2,
            tvoc: payload.tvoc,
            timestamp: payload.timestamp,
            sampling_time,
            state: payload.State.as_deref(),
            mode: payload.Mode.as_deref(),
        };
        self.calc_l1(&p , cfg)
    }
}

pub fn create_l1_calculator(_cfg: &HwInfoDUT) -> Box<dyn DutL1Calculator> {
    Box::new(TempDiffL1::new())
}
