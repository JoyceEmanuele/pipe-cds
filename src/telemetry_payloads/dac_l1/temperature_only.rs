use super::dac_l1_calculator::DacL1Calculator;
use crate::telemetry_payloads::circ_buffer::CircularBuffer;
use chrono::{Duration, NaiveDateTime};

#[derive(Debug)]
struct TelemetryBasicData {
    tsuc: f64,
    tliq: f64,
}

#[derive(Debug, Default)]
pub struct TsucDependentL1 {
    tsuc_memory_filtered: CircularBuffer<901, f64>,
    tliq_memory_filtered: CircularBuffer<901, f64>, //manter o t = -900s na memória facilita muito o código
    tsuc_memory: CircularBuffer<13, f64>,
    tliq_memory: CircularBuffer<13, f64>,
    last_ts: Option<NaiveDateTime>,
    start_ts: Option<NaiveDateTime>,
}

impl TsucDependentL1 {
    pub fn new() -> Self {
        Self::default()
    }

    fn fill_gaps(&mut self, final_ts: NaiveDateTime, telemetry_data: TelemetryBasicData) {
        let amt_secs = match self.last_ts {
            Some(t) => final_ts - t - Duration::seconds(1),
            None => Duration::zero(),
        };

        let base_raw_tsuc = self.tsuc_memory.get(0);
        let base_raw_tliq = self.tliq_memory.get(0);
        let base_filtered_tsuc = self.tsuc_memory_filtered.get(0);
        let base_filtered_tliq = self.tliq_memory_filtered.get(0);

        // unwrap falhar seria o equivalente ao tempo passado ser o mesmo do reset do unix time.
        let amt_seconds: i32 = amt_secs.num_seconds().try_into().unwrap();
        let secs_f64: f64 = f64::try_from(amt_seconds).unwrap();
        // Funções de regressão linear. Adicionam 1 a elapsed por causa do comportamento de fill_with.
        let raw_tsuc_regression = |elapsed: usize| {
            u32::try_from(elapsed)
                .ok()
                .map(f64::from)
                .zip(base_raw_tsuc)
                .map(|(x, base_raw_tsuc)| {
                    base_raw_tsuc + (x + 1.) * (telemetry_data.tsuc - base_raw_tsuc) / secs_f64
                })
        };
        let raw_tliq_regression = |elapsed: usize| {
            u32::try_from(elapsed)
                .ok()
                .map(f64::from)
                .zip(base_raw_tliq)
                .map(|(x, base_raw_tliq)| {
                    base_raw_tliq + (x + 1.) * (telemetry_data.tliq - base_raw_tliq) / secs_f64
                })
        };
        let tsuc_regression = |elapsed: usize| {
            u32::try_from(elapsed)
                .ok()
                .map(f64::from)
                .zip(base_filtered_tsuc)
                .map(|(x, base_raw_tsuc)| {
                    base_raw_tsuc + (x + 1.) * (telemetry_data.tsuc - base_raw_tsuc) / secs_f64
                })
        };
        let tliq_regression = |elapsed: usize| {
            u32::try_from(elapsed)
                .ok()
                .map(f64::from)
                .zip(base_filtered_tliq)
                .map(|(x, base_raw_tliq)| {
                    base_raw_tliq + (x + 1.) * (telemetry_data.tliq - base_raw_tliq) / secs_f64
                })
        };
        self.tsuc_memory.fill_with(
            raw_tsuc_regression,
            amt_secs.num_seconds().try_into().unwrap(),
        );
        self.tliq_memory.fill_with(
            raw_tliq_regression,
            amt_secs.num_seconds().try_into().unwrap(),
        );
        self.tsuc_memory_filtered
            .fill_with(tsuc_regression, amt_secs.num_seconds().try_into().unwrap());
        self.tliq_memory_filtered
            .fill_with(tliq_regression, amt_secs.num_seconds().try_into().unwrap());
    }

    fn reset_memory(&mut self) {
        self.tliq_memory.clear();
        self.tliq_memory_filtered.clear();
        self.tsuc_memory.clear();
        self.tsuc_memory_filtered.clear();
        self.last_ts = None;
        self.start_ts = None;
    }
}

impl DacL1Calculator for TsucDependentL1 {
    fn calc_l1(
        &mut self,
        building_tel: &crate::telemetry_payloads::telemetry_formats::TelemetryDAC_v3,
        full_tel: &crate::telemetry_payloads::telemetry_formats::TelemetryDACv2,
        _cfg: &crate::telemetry_payloads::dac_telemetry::HwInfoDAC,
    ) -> Result<Option<bool>, String> {
        let Some(tamb) = building_tel.Tamb.map(|x| ((10.0 * x).round()) / 10.0) else {
            return Err("No Tamb".into());
        };
        let Some(tsuc) = building_tel.Tsuc.map(|x| ((10.0 * x).round()) / 10.0) else {
            return Err("No Tsuc".into());
        };
        let Some(tliq) = building_tel.Tliq.map(|x| ((10.0 * x).round()) / 10.0) else {
            return Err("No Tliq".into());
        };

        let ts = full_tel.timestamp;
        if let Some(last_ts) = self.last_ts {
            if last_ts >= ts {
                return Err("last_ts >= ts".into());
            }

            // resetar análise
            if ts - last_ts > Duration::minutes(5) {
                self.reset_memory();
            }
        }

        let temps = TelemetryBasicData { tsuc, tliq };

        self.fill_gaps(ts, temps);

        self.last_ts = Some(ts);

        let _ = self.tsuc_memory.insert_point(Some(tsuc));
        let _ = self.tliq_memory.insert_point(Some(tliq));

        let tsuc = self.tsuc_memory.moving_avg(12, 0);
        let tliq = self.tliq_memory.moving_avg(12, 0);

        let _ = self.tsuc_memory_filtered.insert_point(tsuc);
        let _ = self.tliq_memory_filtered.insert_point(tliq);

        let mut conditions = [None, None, None, None, None, None];

        conditions[0] = self
            .tsuc_memory_filtered
            .delta(120)
            .zip(tsuc)
            .map(|(delta, tsuc)| delta > -0.7 && tamb - tsuc < 4.0);

        conditions[1] = self
            .tsuc_memory_filtered
            .delta(60)
            .map(|delta| delta > 0.8)
            .or(Some(false))
            .zip(tliq)
            .map(|(delta_res, tliq)| delta_res && tliq - tamb < 2.5);

        let comparison_deltas = [4 * 60, 6 * 60, 8 * 60, 10 * 60, 12 * 60, 15 * 60];

        conditions[2] = {
            let tliq_deltas = comparison_deltas
                .iter()
                .map(|x| self.tliq_memory_filtered.delta(*x))
                .any(|p| p.map(|p| p < -2.0).unwrap_or(false));

            let delta_tsuc = self
                .tsuc_memory_filtered
                .delta(60)
                .map(|delta| delta > 0.8)
                .unwrap_or(false);

            let diff = tliq.map(|tliq| tliq - tamb >= 2.5);

            diff.map(|diff| delta_tsuc && tliq_deltas && diff)
        };

        conditions[3] = {
            let tsuc_deltas = comparison_deltas
                .iter()
                .map(|x| self.tsuc_memory_filtered.delta(*x))
                .any(|p| p.map(|p| p > 2.0).unwrap_or(false));

            let tsuc_delta = self
                .tsuc_memory_filtered
                .delta(120)
                .map(|x| tsuc_deltas && x >= 0.0)
                .unwrap_or(false);

            let tliq_deltas = comparison_deltas
                .iter()
                .map(|x| self.tliq_memory_filtered.delta(*x))
                .any(|p| p.map(|p| p < -2.0).unwrap_or(false));

            let diff = tliq.map(|tliq| tliq - tamb >= 2.5).unwrap_or(false);

            Some(tsuc_delta && tliq_deltas && diff)
        };

        conditions[4] = {
            let tsuc_delta = self.tsuc_memory_filtered.delta(60).map(|x| x > -0.35);

            tliq.zip(tsuc)
                .zip(tsuc_delta.or(Some(false)))
                .map(|((tliq, tsuc), delta)| delta && tliq - tsuc < 2.0 && tsuc > 20.0)
        };

        conditions[5] = {
            let tsuc_deltas = comparison_deltas
                .iter()
                .map(|x| self.tsuc_memory_filtered.delta(*x))
                .any(|p| p.map(|p| p > 2.0).unwrap_or(false));

            let tsuc_deltas = self
                .tsuc_memory_filtered
                .delta(120)
                .map(|x| tsuc_deltas && x >= 0.0);

            tliq.zip(tsuc_deltas.or(Some(false)))
                .map(|(tliq, delta)| delta && tliq - tamb < 2.5)
        };

        let should_be_off = conditions.into_iter().any(|cond| cond.unwrap_or(false));

        let l1 = !should_be_off;

        match self.start_ts {
            Some(t) if ts - t < Duration::minutes(5) => Ok(None),
            Some(_) => Ok(Some(l1)),
            None => {
                self.start_ts = Some(ts);
                Ok(None)
            }
        }
    }
}
