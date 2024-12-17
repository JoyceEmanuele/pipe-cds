use chrono::{Duration, NaiveDateTime};

use crate::telemetry_payloads::{
    circ_buffer::CircularBuffer, dut_telemetry::HwInfoDUT, telemetry_formats::TelemetryDUTv2,
};

use super::l1_calc::DutL1Calculator;

#[derive(Debug)]
pub struct TempDiffL1 {
    min_tins_off: f64,
    min_tdiff_off: f64,
    lim_dtins_off: f64,
    raw_tins_memory: CircularBuffer<7, f64>,
    raw_tret_memory: CircularBuffer<7, f64>,
    mean_tins_memory: CircularBuffer<216, f64>, // 18 min with data every 5s
    last_valid_timestamp: Option<NaiveDateTime>,
}

impl TempDiffL1 {
    pub fn new() -> Self {
        Self {
            min_tins_off: 28.0,
            min_tdiff_off: 3.0,
            lim_dtins_off: 2.0,
            raw_tins_memory: CircularBuffer::new(),
            raw_tret_memory: CircularBuffer::new(),
            mean_tins_memory: CircularBuffer::new(),
            last_valid_timestamp: None,
        }
    }

    pub fn fill_gaps(&mut self, final_ts: NaiveDateTime) {
        let amt_secs = match self.last_valid_timestamp {
            Some(t) => final_ts - t - Duration::seconds(1),
            None => Duration::zero(),
        };

        let (tins, tret, tins_avg) = if amt_secs <= Duration::seconds(120) {
            (
                self.raw_tins_memory.get(0),
                self.raw_tret_memory.get(0),
                self.mean_tins_memory.get(0),
            )
        } else {
            (None, None, None)
        };
        self.last_valid_timestamp = Some(final_ts);
        for _ in 0..(amt_secs.num_seconds() / 5) {
            self.raw_tins_memory.insert_point(tins);
            self.raw_tret_memory.insert_point(tret);
            self.mean_tins_memory.insert_point(tins_avg);
        }
    }
}

impl DutL1Calculator for TempDiffL1 {
    fn calc_l1(
        &mut self,
        payload: &TelemetryDUTv2,
        _cfg: &HwInfoDUT,
    ) -> Result<Option<bool>, String> {
        let ts = payload.timestamp;

        if self
            .last_valid_timestamp
            .is_some_and(|last_ts| ts <= last_ts)
        {
            return Err("tempo andou para trás".into());
        }

        let tins = payload.temp_1;
        let tret = payload.temp;

        self.fill_gaps(ts);

        self.raw_tins_memory.insert_point(tins);
        self.raw_tret_memory.insert_point(tret);

        let tins_mean = {
            let sum_count = self.raw_tins_memory.iter().fold(None, |sum_count, next| {
                if let Some(v) = next {
                    if let Some((sum, count)) = sum_count {
                        Some((sum + v, count + 1))
                    } else {
                        Some((v, 1))
                    }
                } else {
                    sum_count
                }
            });
            sum_count.map(|(sum, count)| sum / f64::try_from(count).unwrap())
        };

        self.mean_tins_memory.insert_point(tins_mean);

        let tret_mean = self.raw_tret_memory.moving_avg(6, 0);

        let mut conditions = [None, None, None];

        conditions[0] = tret_mean
            .zip(tins_mean)
            .map(|(tret_mean, tins_mean)| (tret_mean - tins_mean) < self.min_tdiff_off);

        let comparison_deltas = [
            4 * 60 / 5,
            6 * 60 / 5,
            8 * 60 / 5,
            10 * 60 / 5,
            12 * 60 / 5,
            15 * 60 / 5,
            18 * 60 / 5,
        ];

        conditions[1] = comparison_deltas
            .iter()
            .map(|x| self.mean_tins_memory.delta(*x))
            .map(|x| x.map(|x| x > self.lim_dtins_off))
            .fold(None, |part, next| match (part, next) {
                (Some(true), _) => Some(true),
                (_, Some(true)) => Some(true),
                (Some(false), _) => Some(false),
                (_, Some(false)) => Some(false),
                _ => None,
            });

        conditions[2] = tins_mean.map(|x| x > self.min_tins_off);

        let should_be_off = conditions
            .into_iter()
            .fold(None, |prev, cond| match (prev, cond) {
                (Some(true), _) => Some(true),
                (_, Some(true)) => Some(true),
                (Some(false), _) => Some(false),
                (_, Some(false)) => Some(false),
                _ => None,
            });

        Ok(should_be_off
            .map(|x| !x)
            .filter(|_| tins.is_some() && tret.is_some()))
    }
}
