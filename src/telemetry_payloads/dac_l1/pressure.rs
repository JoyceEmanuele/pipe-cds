use chrono::{Duration, NaiveDateTime};

use crate::telemetry_payloads::{
    circ_buffer::CircularBuffer,
    dac_telemetry::HwInfoDAC,
    telemetry_formats::{TelemetryDAC_v3, TelemetryDACv2},
};

use super::dac_l1_calculator::DacL1Calculator;

struct TelemetryData {
    tsuc: f64,
    tliq: f64,
    psuc: f64,
}

#[derive(Debug, Default)]
pub(crate) struct PressureBasedL1 {
    psuc_memory: CircularBuffer<{30*60 + 1}, f64>,
    tsuc_memory_filtered: CircularBuffer<{75*60 + 1}, f64>,
    tsuc_memory: CircularBuffer<13, f64>,
    tliq_memory: CircularBuffer<13, f64>,
    pressure_limit1: f64,
    pressure_limit2: f64,
    last_ts: Option<NaiveDateTime>,
    start_ts: Option<NaiveDateTime>,
}

impl PressureBasedL1 {
    pub fn new(cfg: &HwInfoDAC) -> Result<Self, String> {
        let plimit1 = match cfg.fluid.as_deref() {
            Some("r410a" | "r32") => 9.5,
            Some(_) => 6.5,
            None => return Err("Config não tem fluido refrigerante".into()),
        };

        let plimit2 = match cfg.fluid.as_deref() {
            Some("r410a" | "r32") => 8.0,
            Some(_) => 5.5,
            None => return Err("Config não tem fluido refrigerante".into()),
        };

        Ok(Self {
            psuc_memory: CircularBuffer::new(),
            tsuc_memory_filtered: CircularBuffer::new(),
            tsuc_memory: CircularBuffer::new(),
            tliq_memory: CircularBuffer::new(),
            pressure_limit1: plimit1,
            pressure_limit2: plimit2,
            last_ts: None,
            start_ts: None,
        })
    }
    fn fill_gaps(&mut self, final_ts: NaiveDateTime, telemetry_data: TelemetryData) {
        let amt_secs = match self.last_ts {
            Some(t) => final_ts - t - Duration::seconds(1),
            None => Duration::zero(),
        };

        let base_raw_tsuc = self.tsuc_memory.get(0);
        let base_raw_tliq = self.tliq_memory.get(0);
        let base_raw_psuc = self.psuc_memory.get(0);
        let base_filtered_tsuc = self.tsuc_memory_filtered.get(0);

        let amt_seconds: i32 = amt_secs.num_seconds().try_into().unwrap();
        let secs_f64: f64 = f64::from(amt_seconds);
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
        let psuc_regression = |elapsed: usize| {
            u32::try_from(elapsed)
                .ok()
                .map(f64::from)
                .zip(base_raw_psuc)
                .map(|(x, base_raw_psuc)| {
                    base_raw_psuc + (x + 1.) * (telemetry_data.psuc - base_raw_psuc) / secs_f64
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
        self.psuc_memory
            .fill_with(psuc_regression, amt_secs.num_seconds().try_into().unwrap());

        self.last_ts = Some(final_ts);
    }

    fn reset_memory(&mut self) {
        self.tliq_memory.clear();
        self.psuc_memory.clear();
        self.tsuc_memory.clear();
        self.tsuc_memory_filtered.clear();
        self.last_ts = None;
        self.start_ts = None;
    }
}

impl DacL1Calculator for PressureBasedL1 {
    fn calc_l1(
        &mut self,
        building_tel: &TelemetryDAC_v3,
        tel: &TelemetryDACv2,
        cfg: &HwInfoDAC,
    ) -> Result<Option<bool>, String> {
        let ts = tel.timestamp;
        if let Some(last_ts) = self.last_ts {
            if last_ts >= ts {
                return Err("last_ts < ts".into());
            }

            // resetar análise
            if ts - last_ts > Duration::minutes(5) {
                self.reset_memory();
            }
        }

        if building_tel.Tsuc.is_none() || building_tel.Psuc.is_none() {
            return Err("No data".into());
        }

        let Some(tsuc) = building_tel.Tsuc else {
            return Err("No Tsuc".into());
        };
        let Some(psuc) = building_tel.Psuc else {
            return Err("No Psuc".into());
        };
        let Some(tamb) = building_tel.Tamb else {
            return Err("No Tamb".into());
        };
        let Some(tliq) = building_tel.Tliq else {
            return Err("No Tliq".into());
        };

        let telemetry_data = TelemetryData { tsuc, psuc, tliq };

        self.fill_gaps(ts, telemetry_data);

        let _ = self.tsuc_memory.insert_point(Some(tsuc));
        let _ = self.psuc_memory.insert_point(Some(psuc));
        let _ = self.tliq_memory.insert_point(Some(tliq));

        let psuc_adc = if cfg.P0Psuc {
            tel.p0
        } else if cfg.P1Psuc {
            tel.p1
        } else {
            None
        };

        if psuc_adc.map(|p| p < 70).unwrap_or(true) {
            return Err("invalid psuc adc".into());
        }

        let tsuc = self.tsuc_memory.moving_avg(12, 0);
        let _ = self.tsuc_memory_filtered.insert_point(tsuc);
        let psuc = self.psuc_memory.moving_avg(12, 0);
        let tliq = self.tliq_memory.moving_avg(12, 0);

        let mut conditions: [Option<bool>; 14] = [None; 14];

        let tamb_m_tsuc = tsuc.map(|x| tamb - x);
        let tliq_m_tamb = tliq.map(|x| x - tamb);
        let tliq_m_tsuc = tliq.zip(tsuc).map(|(x, y)| x - y);
        let psuc_gt_plimit1 = psuc.map(|x| x > self.pressure_limit1);
        let psuc_gt_plimit2 = psuc.map(|x| x > self.pressure_limit2);
        let dtsuc_30s = self.tsuc_memory_filtered.delta(30);
        let dtsuc_60s = self.tsuc_memory_filtered.delta(60);
        let dtsuc_120s = self.tsuc_memory_filtered.delta(120);
        let dtsuc_170s = self.tsuc_memory_filtered.delta(170);
        let dpsuc_15s = self.psuc_memory.delta(15);
        let dpsuc_240s = self.psuc_memory.delta(240);
        let dpsuc_600s = self.psuc_memory.delta(600);
        let dpsuc_30m = self.psuc_memory.delta(30*60);

        conditions[0] = dtsuc_60s
            .zip(tsuc)
            .zip(tliq)
            .zip(psuc)
            .map(|(((delta, tsuc), tliq), psuc)| {
                delta > -0.35 && (tamb - tsuc) < 2.5 && (tliq - tamb < 4.0) && psuc > self.pressure_limit1
            });

        conditions[1] = dtsuc_170s
            .zip(tamb_m_tsuc)
            .zip(psuc_gt_plimit1)
            .zip(tliq_m_tsuc)
            .map(|(((delta, tamb_m_tsuc), psuc_gt_plimit1), tliq_m_tsuc)| {
                delta >= -0.4 && psuc_gt_plimit1 && (tamb_m_tsuc < 8.0) && (tliq_m_tsuc < 16.0)
            });

        conditions[2] = dpsuc_15s
            .zip(dpsuc_600s)
            .zip(dtsuc_60s)
            .zip(tamb_m_tsuc)
            .zip(tliq_m_tsuc)
            .map(
                |((((delta15s_psuc, delta600s_psuc), delta_60s_tsuc), tamb_m_tsuc), tliq_m_tsuc)| {
                    (delta15s_psuc > 2.5 || delta600s_psuc > 0.4)
                        && (delta_60s_tsuc > 0.8 || tamb_m_tsuc < 2.5)
                        && tamb_m_tsuc < 16.0
                        && tliq_m_tsuc < 16.0
                },
            );

        conditions[3] = dtsuc_120s
            .zip(psuc_gt_plimit1)
            .zip(tamb_m_tsuc)
            .zip(tliq_m_tsuc)
            .map(|(((dtsuc_120, psuc_gt_plimit), tamb_m_tsuc), tliq_m_tsuc)| {
                dtsuc_120 > 0.4 && psuc_gt_plimit && tamb_m_tsuc < 16.0 && tliq_m_tsuc < 16.0 
            });

        conditions[4] = dtsuc_60s
            .zip(psuc_gt_plimit1)
            .zip(tamb_m_tsuc)
            .zip(tliq_m_tsuc)
            .map(|(((dt60, psuc_gt_plimit1), tamb_m_tsuc), tliq_m_tsuc)| {
                dt60 > -0.9 && psuc_gt_plimit1 && tamb_m_tsuc < 16.0 && tliq_m_tsuc < 16.0
            });

        conditions[5] = dtsuc_60s
            .zip(tsuc)
            .zip(tliq)
            .map(|((dt60, tsuc), tliq)| {
                dt60 > -0.35 && (tliq - tsuc < 2.0) && tsuc > 20.0 && tamb - tsuc < 16.0 && tliq - tamb < 16.0
            });

        conditions[6] = dtsuc_60s
            .zip(tamb_m_tsuc)
            .zip(tliq_m_tamb)
            .map(|((dt60, tamb_m_tsuc), tliq_m_tamb)| {
                dt60 > -0.35 && (
                    (tamb_m_tsuc < 4.5 && tliq_m_tamb < -1.2)
                    || (tamb_m_tsuc < 1.0 && tliq_m_tamb < 3.8)
                )
            });
        conditions[7] = dtsuc_60s
            .zip(tamb_m_tsuc)
            .zip(tliq_m_tamb)
            .map(|((dt60, tamb_m_tsuc), tliq_m_tamb)| {
                dt60 > -0.35 && tamb_m_tsuc < 1.5 && tliq_m_tamb < 3.8
            });
        

        let deltas_condition = [
            2*60_usize,
            3*60_usize,
            5*60_usize,
            8*60_usize,
            10*60_usize,
            15*60_usize,
            30*60_usize,
            45*60_usize,
            75*60_usize
        ]
        .map(|x| self.tsuc_memory_filtered.delta(x))
        .map(|x| x.map(|y| y > 6.0))
        .into_iter()
        .fold(None, |prev, cond| match (prev, cond) {
            (Some(true), _) => Some(true),
            (_, Some(true)) => Some(true),
            (Some(false), _) => Some(false),
            (_, Some(false)) => Some(false),
            _ => None,
        });


        conditions[8] = deltas_condition
            .zip(dtsuc_30s)
            .zip(dpsuc_240s)
            .zip(tliq_m_tamb)
            .map(|(((deltas, dt_30), dp_240), tliq_m_tamb)| {
                deltas && dt_30 >= -1.3 && dp_240 <= -3.5 && tliq_m_tamb < 6.5
            });
        
        conditions[9] = dpsuc_600s
            .zip(dpsuc_30m)
            .map(|(dp_10m, dp_30m)| {
                dp_10m > 5.0 || dp_30m > 5.0
            });

        let deltas_condition = [
            2*60_usize,
            4*60_usize,
            6*60_usize,
            8*60_usize,
            10*60_usize
        ].map(|x| self.tsuc_memory_filtered.delta(x))
        .map(|x| x.map(|y| y > 6.0))
        .into_iter()
        .fold(None, |prev, cond| match (prev, cond) {
            (Some(true), _) => Some(true),
            (_, Some(true)) => Some(true),
            (Some(false), _) => Some(false),
            (_, Some(false)) => Some(false),
            _ => None,
        });
        
        conditions[10] = deltas_condition
            .zip(psuc_gt_plimit2)
            .map(|(deltas, psuc_gt_p2)| {
                deltas && psuc_gt_p2
            });

        conditions[11] = tsuc
            .zip(tliq)
            .map(|(tsuc, tliq)| {
                (tsuc > tliq || tsuc > tamb) && tsuc > 21.0 && (tliq - tamb < 4.5) && (tliq - tsuc < 4.5)
            });
        conditions[12] = dtsuc_60s
            .zip(tliq)
            .zip(tsuc)
            .map(|((dt_60, tliq), tsuc)| {
                dt_60 > -0.35 && (tamb - tsuc < 4.0) && (tliq - tamb < 4.5) && (tliq - tsuc < 4.5) 
            });
        conditions[13] = psuc
            .zip(tliq)
            .zip(tsuc)
            .map(|((psuc, tliq), tsuc)| {
                tamb - tsuc < 0.5 && tliq - tamb < 9.5 && psuc < 1.0
            });

        let should_be_off = conditions
            .into_iter()
            .fold(None, |prev, cond| match (prev, cond) {
                (Some(true), _) => Some(true),
                (_, Some(true)) => Some(true),
                (Some(false), _) => Some(false),
                (_, Some(false)) => Some(false),
                _ => None,
            });

        let l1 = should_be_off.map(|x| !x);

        match self.start_ts {
            Some(t) if ts - t < Duration::minutes(5) => Ok(None),
            Some(_) => Ok(l1),
            None => {
                self.start_ts = Some(ts);
                Ok(None)
            }
        }
    }
}
