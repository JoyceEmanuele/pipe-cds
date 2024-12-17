use std::ops::DerefMut;

use chrono::{Duration, NaiveDateTime};

use crate::telemetry_payloads::{circ_buffer::CircularBuffer, dac_l1::temp_difference::TemperatureDifferenceL1, dac_telemetry::HwInfoDAC, telemetry_formats::{TelemetryDAC_v3, TelemetryDACv2}};

use super::{fancoil::DacL1Fancoil, no_tsuc::NoTsucL1, physical::PhysicalL1, pressure::PressureBasedL1, temperature_only::TsucDependentL1};

pub trait DacL1Calculator: Send + Sync {
    /// Esse trait vai assumir que Tamb, Tsuc, Tliq, Psuc e Pliq de `building_tel` já estão inicializados.
    fn calc_l1(
        &mut self,
        building_tel: &TelemetryDAC_v3,
        full_tel: &TelemetryDACv2,
        cfg: &HwInfoDAC,
    ) -> Result<Option<bool>, String>;
}

/// Torna qualquer coisa que dereferencia para [D], tal como Vec, um DacL1Calculator de forma que o L1 será avaliado em ordem.
/// Os L1s subsequentes serão usados para tapar os buracos dos anteriores (por conta de falta de dados, etc.)
/// D é algo que age como &mut dyn DacL1Calculator
impl<T, D> DacL1Calculator for T
where
    T: DerefMut<Target = [D]> + Send + Sync,
    D: DerefMut<Target = dyn DacL1Calculator>,
{
    fn calc_l1(
        &mut self,
        building_tel: &TelemetryDAC_v3,
        tel: &TelemetryDACv2,
        cfg: &HwInfoDAC,
    ) -> Result<Option<bool>, String> {
        let mut fallback_checkers = (*self).iter_mut();
        let first_calc = fallback_checkers
            .next()
            .ok_or_else(|| "Não há checker de l1".to_string())?;

        let mut l1 = first_calc.calc_l1(building_tel, tel, cfg).ok().flatten();

        for checker in fallback_checkers {
            let Ok(fallback_l1) = checker.calc_l1(building_tel, tel, cfg) else {
                continue;
            };
            if l1.is_none() {
                let _ = std::mem::replace(&mut l1, fallback_l1);
            }
        }

        Ok(l1)
    }
}

impl DacL1Calculator for Box<dyn DacL1Calculator> {
    fn calc_l1(
        &mut self,
        building_tel: &TelemetryDAC_v3,
        full_tel: &TelemetryDACv2,
        cfg: &HwInfoDAC,
    ) -> Result<Option<bool>, String> {
        let c = self.as_mut();
        c.calc_l1(building_tel, full_tel, cfg)
    }
}

pub struct DacVirtualCalculator<T> {
    calc: T,
    unfiltered_l1: CircularBuffer<30, bool>,
    last_ts: Option<NaiveDateTime>,
}

impl<T> DacVirtualCalculator<T> {
    fn fill_gaps(&mut self, final_ts: NaiveDateTime) {
        let amt_secs = match self.last_ts {
            Some(t) => final_ts - t - Duration::seconds(1),
            None => Duration::zero(),
        };
        //fill with neighbor for small time increments
        let l1_to_fill = if amt_secs <= Duration::seconds(15) {
            self.unfiltered_l1.get(0)
        } else {
            None
        };

        for _ in 0..amt_secs.num_seconds() {
            self.unfiltered_l1.insert_point(l1_to_fill);
        }
        self.last_ts = Some(final_ts);
    }
}

impl<T: DacL1Calculator> DacL1Calculator for DacVirtualCalculator<T> {
    fn calc_l1(
        &mut self,
        building_tel: &TelemetryDAC_v3,
        full_tel: &TelemetryDACv2,
        cfg: &HwInfoDAC,
    ) -> Result<Option<bool>, String> {
        let ts = full_tel.timestamp;
        if let Some(last_ts) = self.last_ts {
            if last_ts >= ts {
                return Err("last_ts >= ts".into());
            }
        }
        self.fill_gaps(ts);

        let l1 = self
            .calc
            .calc_l1(building_tel, full_tel, cfg)
            .ok()
            .flatten();
        self.unfiltered_l1.insert_point(l1);

        // Unwraps existem pois o MAX_SIZE de self.unfiltered_l1 é pequeno então a conversão nunca falha.
        let l1_on = f64::from(u32::try_from(self.unfiltered_l1.entries_matching(&true)).unwrap());

        let valid_l1 = f64::from(u32::try_from(self.unfiltered_l1.valid_entries()).unwrap());

        let keep_l1_threshold = valid_l1 / 2.0;

        let filtered_l1 = if keep_l1_threshold > 0.0 {
            Some(l1_on > keep_l1_threshold)
        } else {
            None
        };

        Ok(filtered_l1)
    }
}

pub fn create_l1_calculator(cfg: &HwInfoDAC) -> Box<dyn DacL1Calculator> {
    if cfg.isVrf || cfg.simulate_l1 {
        let v: Vec<Box<dyn DacL1Calculator>> = if cfg.P0Psuc || cfg.P1Psuc {
            let pressure_l1 = PressureBasedL1::new(cfg);
            let default_calcs: [Box<dyn DacL1Calculator>; 3] = [
                Box::new(TsucDependentL1::new()),
                Box::new(TemperatureDifferenceL1 {}),
                Box::new(NoTsucL1 {}),
            ];
            pressure_l1
                .into_iter()
                .map(|x| Box::new(x) as Box<dyn DacL1Calculator>)
                .chain(default_calcs)
                .collect::<Vec<Box<dyn DacL1Calculator>>>()
        } else {
            vec![
                Box::new(TsucDependentL1::new()),
                Box::new(TemperatureDifferenceL1 {}),
                Box::new(NoTsucL1 {}),
            ]
        };
        Box::new(DacVirtualCalculator {
            calc: v,
            unfiltered_l1: CircularBuffer::new(),
            last_ts: None,
        })
    } else if cfg.calculate_L1_fancoil.unwrap_or(false) {
        Box::new(DacL1Fancoil {})
    } else {
        Box::new(PhysicalL1 {})
    }
}

pub fn should_update_l1_calc(last_cfg: Option<&HwInfoDAC>, new_cfg: &HwInfoDAC) -> bool {
    let Some(last_cfg) = last_cfg else {
        return true;
    };
    let virtual_l1_state_changed = last_cfg.simulate_l1 != new_cfg.simulate_l1;
    let vrf_state_changed = last_cfg.isVrf != new_cfg.isVrf;
    let fancoil_state_changed =
        last_cfg.calculate_L1_fancoil != new_cfg.calculate_L1_fancoil && !new_cfg.isVrf;
    let pressure_state_changed =
        (last_cfg.P0Psuc || last_cfg.P1Psuc) != (new_cfg.P0Psuc || new_cfg.P1Psuc);
    let fluid_changed = last_cfg.fluid != new_cfg.fluid;

    virtual_l1_state_changed
        || vrf_state_changed
        || fancoil_state_changed
        || (new_cfg.isVrf && pressure_state_changed)
        || fluid_changed
}
