use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use serde_json::{json, Value};

use crate::telemetry_payloads::energy::dme::TelemetryDME;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct PadronizedEnergyTelemetry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v_a: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v_b: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v_c: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v_ab: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v_bc: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v_ca: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub i_a: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub i_b: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub i_c: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pot_at_a: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pot_at_b: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pot_at_c: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pot_ap_a: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pot_ap_b: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pot_ap_c: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pot_re_a: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pot_re_b: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pot_re_c: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v_tri_ln: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v_tri_ll: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pot_at_tri: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pot_ap_tri: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pot_re_tri: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub en_at_tri: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub en_re_tri: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub en_ap_tri: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fp_a: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fp_b: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fp_c: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fp: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freq: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub demanda_at: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub demanda_ap: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub demanda_med_at: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub demanda: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub erro: Option<f64>,
    pub timestamp: Option<NaiveDateTime>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct EnergyDemandTelemetry {
    pub demanda_med_at: f64,
    pub min_demand: f64,
    pub max_demand: f64,
    pub timestamp: NaiveDateTime,
}

impl EnergyDemandTelemetry {
    pub fn set_average_demand(&mut self, value: f64) {
        self.demanda_med_at = value;
    }

    pub fn new(timestamp: NaiveDateTime) -> Self {
        Self {
            timestamp,
            demanda_med_at: 0.0,
            min_demand: 0.0,
            max_demand: 0.0,
        }
    }

    pub fn set_min_demand(&mut self, value: f64) {
        self.min_demand = value;
    }

    pub fn set_max_demand(&mut self, value: f64) {
        self.max_demand = value;
    }
}


pub fn calculate_formulas(param: &str, value: f64, tel: &Value, is_ieee754_fp: bool) -> f64 {
    let computed_value = {
        if is_ieee754_fp {
            return f32::from_bits(value as u32) as f64;
        }
        value
    };

    let formula = tel.get("formulas")
        .and_then(|f| f.as_object())
        .and_then(|f| f.get(param))
        .and_then(|f| f.as_str());

    if formula.is_none() {
        return computed_value;
    }

    let formulaString = formula.unwrap();
    let variables: Vec<String> = formulaString.match_indices("CMN").map(|(i, _)| {
        let mut result = String::from("CMN");
        for c in formulaString.get(i+3..).unwrap().chars() {
            if !c.is_numeric() {
                break;
            }
            result.push_str(c.to_string().as_str());
        }
        result
    }).collect();

    let expr: meval::Expr = format!("X{}", formulaString).parse().unwrap();
    if variables.is_empty() {
        if let Ok(func) = expr.clone().bind("X") {
            return func(computed_value);
        }
    }
    if variables.len() == 1 {
        let var = "X";
        let var0 = variables[0].as_str();
        let var0_value = tel.get(var0).and_then(|f| f.as_f64());
        if var0_value.is_none() {
            if (param == "en_at_tri") {
                return -1.0;
            } else {
                return computed_value;
            }
        }
        let var0_value_calc = calculate_formulas(var0, var0_value.unwrap(), &tel, is_ieee754_fp);
        if let Ok(func) = expr.clone().bind2(var, var0) {
            return func(computed_value, var0_value_calc);
        }
    }
    if variables.len() == 2 {
        let var = "X";
        let var0 = variables[0].as_str();
        let var0_value = tel.get(var0).and_then(|f| f.as_f64());
        let var1 = variables[1].as_str();
        let var1_value = tel.get(var1).and_then(|f| f.as_f64());
        if var0_value.is_none() || var1_value.is_none() {
            if param == "en_at_tri" {
                return -1.0;
            } else {
                return computed_value;
            }
        }
        let var0_value_calc = calculate_formulas(var0, var0_value.unwrap(), &tel, is_ieee754_fp);
        let var1_value_calc = calculate_formulas(var1, var1_value.unwrap(), &tel, is_ieee754_fp);
        if let Ok(func) = expr.clone().bind3(var, var0, var1) {
            return func(computed_value, var0_value_calc, var1_value_calc);
        }
    }
    if variables.len() == 3 {
        let var = "X";
        let var0 = variables[0].as_str();
        let var0_value = tel.get(var0).and_then(|f| f.as_f64());
        let var1 = variables[1].as_str();
        let var1_value = tel.get(var1).and_then(|f| f.as_f64());
        let var2 = variables[2].as_str();
        let var2_value = tel.get(var2).and_then(|f| f.as_f64());

        if var0_value.is_none() || var1_value.is_none() || var2_value.is_none() {
            if param == "en_at_tri" {
                return -1.0;
            } else {
                return computed_value;
            }
        }
        let var0_value_calc = calculate_formulas(var0, var0_value.unwrap(), tel, is_ieee754_fp);
        let var1_value_calc = calculate_formulas(var1, var1_value.unwrap(), tel, is_ieee754_fp);
        let var2_value_calc = calculate_formulas(var2, var2_value.unwrap(), tel, is_ieee754_fp);
        if let Ok(func) = expr.bind4(var, var0, var1, var2) {
            return func(computed_value, var0_value_calc, var1_value_calc, var2_value_calc);
        }
    }

    computed_value
}

fn check_parameter(value: Option<f64>, options: &Vec<String>, option: &String) -> Option<f64> {
    if options.contains(option) {
        return value
    }
    return None
}

fn convert_4Q_FP_PF(value: f64) -> f64 {
    if value > 1.0 {
        return 2.0 - value;
    } else if value < -1.0 {
        return -2.0 - value;
    } else if value.abs() == 1.0 {
        return value;
    }
    value
}


pub fn format_padronized_energy_temeletry (tel: PadronizedEnergyTelemetry, options: Option<&Vec<String>>) -> PadronizedEnergyTelemetry {
    if options.is_none() {
        return tel
    }
    if options.unwrap().is_empty() {
        return tel
    }
    PadronizedEnergyTelemetry {
        timestamp: tel.timestamp,
        v_a: check_parameter(tel.v_a, &options.as_ref().unwrap(), &"v_a".to_string()),
        v_b: check_parameter(tel.v_b, &options.as_ref().unwrap(), &"v_b".to_string()),
        v_c: check_parameter(tel.v_c, &options.as_ref().unwrap(), &"v_c".to_string()),
        v_ab: check_parameter(tel.v_ab, &options.as_ref().unwrap(), &"v_ab".to_string()),
        v_bc: check_parameter(tel.v_bc, &options.as_ref().unwrap(), &"v_bc".to_string()),
        v_ca: check_parameter(tel.v_ca, &options.as_ref().unwrap(), &"v_ca".to_string()),
        i_a: check_parameter(tel.i_a, &options.as_ref().unwrap(), &"i_a".to_string()),
        i_b: check_parameter(tel.i_b, &options.as_ref().unwrap(), &"i_b".to_string()),
        i_c: check_parameter(tel.i_c, &options.as_ref().unwrap(), &"i_c".to_string()),
        pot_at_a: check_parameter(tel.pot_at_a, &options.as_ref().unwrap(), &"pot_at_a".to_string()),
        pot_at_b: check_parameter(tel.pot_at_b, &options.as_ref().unwrap(), &"pot_at_b".to_string()),
        pot_at_c: check_parameter(tel.pot_at_c, &options.as_ref().unwrap(), &"pot_at_c".to_string()),
        pot_ap_a: check_parameter(tel.pot_ap_a, &options.as_ref().unwrap(), &"pot_ap_a".to_string()),
        pot_ap_b: check_parameter(tel.pot_ap_b, &options.as_ref().unwrap(), &"pot_ap_b".to_string()),
        pot_ap_c: check_parameter(tel.pot_ap_c, &options.as_ref().unwrap(), &"pot_ap_c".to_string()),
        pot_re_a: check_parameter(tel.pot_re_a, &options.as_ref().unwrap(), &"pot_re_a".to_string()),
        pot_re_b: check_parameter(tel.pot_re_b, &options.as_ref().unwrap(), &"pot_re_b".to_string()),
        pot_re_c: check_parameter(tel.pot_re_c, &options.as_ref().unwrap(), &"pot_re_c".to_string()),
        v_tri_ln: check_parameter(tel.v_tri_ln, &options.as_ref().unwrap(), &"v_tri_ln".to_string()),
        v_tri_ll: check_parameter(tel.v_tri_ll, &options.as_ref().unwrap(), &"v_tri_ll".to_string()),
        pot_at_tri: check_parameter(tel.pot_at_tri, &options.as_ref().unwrap(), &"pot_at_tri".to_string()),
        pot_ap_tri: check_parameter(tel.pot_ap_tri, &options.as_ref().unwrap(), &"pot_ap_tri".to_string()),
        pot_re_tri: check_parameter(tel.pot_re_tri, &options.as_ref().unwrap(), &"pot_re_tri".to_string()),
        en_at_tri: check_parameter(tel.en_at_tri, &options.as_ref().unwrap(), &"en_at_tri".to_string()),
        en_re_tri: check_parameter(tel.en_re_tri, &options.as_ref().unwrap(), &"en_re_tri".to_string()),
        fp_a: check_parameter(tel.fp_a, &options.as_ref().unwrap(), &"fp_a".to_string()),
        fp_b: check_parameter(tel.fp_b, &options.as_ref().unwrap(), &"fp_b".to_string()),
        fp_c: check_parameter(tel.fp_c, &options.as_ref().unwrap(), &"fp_c".to_string()),
        fp: check_parameter(tel.fp, &options.as_ref().unwrap(), &"fp".to_string()),
        freq: check_parameter(tel.freq, &options.as_ref().unwrap(), &"freq".to_string()),
        demanda: check_parameter(tel.demanda, &options.as_ref().unwrap(), &"demanda".to_string()),
        demanda_at: check_parameter(tel.demanda_at, &options.as_ref().unwrap(), &"demanda_at".to_string()),
        demanda_ap: check_parameter(tel.demanda_ap, &options.as_ref().unwrap(), &"demanda_ap".to_string()),
        demanda_med_at: check_parameter(tel.demanda_med_at, &options.as_ref().unwrap(), &"demanda_med_at".to_string()),
        erro: check_parameter(tel.erro, &options.as_ref().unwrap(), &"erro".to_string()),
        en_ap_tri: check_parameter(tel.en_ap_tri, &options.as_ref().unwrap(), &"en_ap_tri".to_string()),
    }
}

impl<'a> TryFrom<TelemetryDME<'a>> for PadronizedEnergyTelemetry {
    type Error = String;
    fn try_from(value: TelemetryDME) -> Result<PadronizedEnergyTelemetry, String> {
        let timestamp = NaiveDateTime::parse_from_str(value.timestamp.as_ref(), "%Y-%m-%dT%H:%M:%S")
            .map_err(|e| e.to_string())?;
        let tel = json!(value);

        let is_schneider_pm2100 = match value.dev_type.as_ref() {
            Some(v) => v.as_str() == "SCHNEIDER-ELETRIC-PM2100",
            _ => false,
        };

        let is_schneider_pm210 = match value.dev_type.as_ref() {
            Some(v) => v.as_str() == "SCHNEIDER-ELECTRIC-PM210",
            _ => false,
        };

        let is_ieee754_fp = match value.dev_type.as_ref() {
            Some(v) => v.as_str() == "KRON-IKRON-03" || is_schneider_pm2100 || is_schneider_pm210,
            _ => false,
        };

        let result = PadronizedEnergyTelemetry {
            timestamp: Some(timestamp),
            v_a: match value.v_a {
                None => None,
                Some(-1.0) => None,
                _ => Some(calculate_formulas("v_a", value.v_a.unwrap(), &tel, is_ieee754_fp)),
            },
            v_b: match value.v_b {
                None => None,
                Some(-1.0) => None,
                _ => Some(calculate_formulas("v_b", value.v_b.unwrap(), &tel, is_ieee754_fp)),
            },
            v_c: match value.v_c {
                None => None,
                Some(-1.0) => None,
                _ => Some(calculate_formulas("v_c", value.v_c.unwrap(), &tel, is_ieee754_fp)),
            },
            v_ab: match value.v_ab {
                None => None,
                Some(-1.0) => None,
                _ => Some(calculate_formulas("v_ab", value.v_ab.unwrap(), &tel, is_ieee754_fp)),
            },
            v_bc: match value.v_bc {
                None => None,
                Some(-1.0) => None,
                _ => Some(calculate_formulas("v_bc", value.v_bc.unwrap(), &tel, is_ieee754_fp)),
            },
            v_ca: match value.v_ca {
                None => None,
                Some(-1.0) => None,
                _ => Some(calculate_formulas("v_ca", value.v_ca.unwrap(), &tel, is_ieee754_fp)),
            },
            i_a: match value.i_a {
                None => None,
                Some(-1.0) => None,
                _ => Some(calculate_formulas("i_a", value.i_a.unwrap(), &tel, is_ieee754_fp)),
            },
            i_b: match value.i_b {
                None => None,
                Some(-1.0) => None,
                _ => Some(calculate_formulas("i_b", value.i_b.unwrap(), &tel, is_ieee754_fp)),
            },
            i_c: match value.i_c {
                None => None,
                Some(-1.0) => None,
                _ => Some(calculate_formulas("i_c", value.i_c.unwrap(), &tel, is_ieee754_fp)),
            },
            pot_at_a: match value.pot_at_a {
                None => None,
                Some(-1.0) => None,
                _ => Some(calculate_formulas("pot_at_a", value.pot_at_a.unwrap(), &tel, is_ieee754_fp)),
            },
            pot_at_b: match value.pot_at_b {
                None => None,
                Some(-1.0) => None,
                _ => Some(calculate_formulas("pot_at_b", value.pot_at_b.unwrap(), &tel, is_ieee754_fp)),
            },
            pot_at_c: match value.pot_at_c {
                None => None,
                Some(-1.0) => None,
                _ => Some(calculate_formulas("pot_at_c", value.pot_at_c.unwrap(), &tel, is_ieee754_fp)),
            },
            pot_ap_a: match value.pot_ap_a {
                None => None,
                Some(-1.0) => None,
                _ => Some(calculate_formulas("pot_ap_a", value.pot_ap_a.unwrap(), &tel, is_ieee754_fp)),
            },
            pot_ap_b: match value.pot_ap_b {
                None => None,
                Some(-1.0) => None,
                _ => Some(calculate_formulas("pot_ap_b", value.pot_ap_b.unwrap(), &tel, is_ieee754_fp)),
            },
            pot_ap_c: match value.pot_ap_c {
                None => None,
                Some(-1.0) => None,
                _ => Some(calculate_formulas("pot_ap_c", value.pot_ap_c.unwrap(), &tel, is_ieee754_fp)),
            },
            pot_re_a: match value.pot_re_a {
                None => None,
                Some(-1.0) => None,
                _ => Some(calculate_formulas("pot_re_a", value.pot_re_a.unwrap(), &tel, is_ieee754_fp)),
            },
            pot_re_b: match value.pot_re_b {
                None => None,
                Some(-1.0) => None,
                _ => Some(calculate_formulas("pot_re_b", value.pot_re_b.unwrap(), &tel, is_ieee754_fp)),
            },
            pot_re_c: match value.pot_re_c {
                None => None,
                Some(-1.0) => None,
                _ => Some(calculate_formulas("pot_re_c", value.pot_re_c.unwrap(), &tel, is_ieee754_fp)),
            },
            v_tri_ln: match value.v_tri_ln {
                None => None,
                Some(-1.0) => None,
                _ => Some(calculate_formulas("v_tri_ln", value.v_tri_ln.unwrap(), &tel, is_ieee754_fp)),
            },
            v_tri_ll: match value.v_tri_ll {
                None => None,
                Some(-1.0) => None,
                _ => Some(calculate_formulas("v_tri_ll", value.v_tri_ll.unwrap(), &tel, is_ieee754_fp)),
            },
            pot_at_tri: match value.pot_at_tri {
                None => None,
                Some(-1.0) => None,
                _ => Some(calculate_formulas("pot_at_tri", value.pot_at_tri.unwrap(), &tel, is_ieee754_fp)),
            },
            pot_ap_tri: match value.pot_ap_tri {
                None => None,
                Some(-1.0) => None,
                _ => Some(calculate_formulas("pot_ap_tri", value.pot_ap_tri.unwrap(), &tel, is_ieee754_fp)),
            },
            pot_re_tri: match value.pot_re_tri {
                None => None,
                Some(-1.0) => None,
                _ => Some(calculate_formulas("pot_re_tri", value.pot_re_tri.unwrap(), &tel, is_ieee754_fp)),
            },
            en_at_tri: match value.en_at_tri {
                None => None,
                Some(-1.0) => None,
                _ => {
                    if value.en_at_tri == Some(65535.0) || value.en_at_tri == Some(1845494299.0) || value.en_at_tri == Some(2147483647.0) {
                        Some(-1.0)
                    } else {
                        let result = if is_schneider_pm2100 {
                            calculate_formulas("en_at_tri", value.en_at_tri.unwrap(), &tel, false)
                        } else {
                            calculate_formulas("en_at_tri", value.en_at_tri.unwrap(), &tel, is_ieee754_fp)
                        };
                        
                        let final_result = if result < 0.0 {
                            Some(-1.0)
                        } else {
                            Some(result)
                        };

                        final_result
                    }
                },
            },
            en_re_tri: match value.en_re_tri {
                None => None,
                Some(-1.0) => None,
                _ => {
                    if is_schneider_pm2100 {
                        Some(calculate_formulas("en_re_tri", value.en_re_tri.unwrap(), &tel, false))
                    } else  {
                        Some(calculate_formulas("en_re_tri", value.en_re_tri.unwrap(), &tel, is_ieee754_fp))
                    }
                }
            },
            en_ap_tri: match value.en_ap_tri {
                None => None,
                Some(-1.0) => None,
                _ => {
                    Some(calculate_formulas("en_ap_tri", value.en_ap_tri.unwrap(), &tel, is_ieee754_fp))
                }
            },
            fp_a: match value.fp_a {
                None => None,
                Some(-1.0) => None,
                _ => {
                    let value = calculate_formulas("fp_a", value.fp_a.unwrap(), &tel, is_ieee754_fp);
                    if is_schneider_pm2100 {
                        Some(convert_4Q_FP_PF(value))
                    } else {
                        Some(value)
                    }
                }
            },
            fp_b: match value.fp_b {
                None => None,
                Some(-1.0) => None,
                _ => {
                    let value = calculate_formulas("fp_b", value.fp_b.unwrap(), &tel, is_ieee754_fp);
                    if is_schneider_pm2100 {
                        Some(convert_4Q_FP_PF(value))
                    } else {
                        Some(value)
                    }
                }            },
            fp_c: match value.fp_c {
                None => None,
                Some(-1.0) => None,
                _ => {
                    let value = calculate_formulas("fp_c", value.fp_c.unwrap(), &tel, is_ieee754_fp);
                    if is_schneider_pm2100 {
                        Some(convert_4Q_FP_PF(value))
                    } else {
                        Some(value)
                    }
                }
            },
            fp: match value.fp {
                None => None,
                Some(-1.0) => None,
                _ => {
                    let value = calculate_formulas("fp", value.fp.unwrap(), &tel, is_ieee754_fp);
                    if is_schneider_pm2100 {
                        Some(convert_4Q_FP_PF(value))
                    } else {
                        Some(value)
                    }
                }            },
            freq: match value.freq {
                None => None,
                Some(-1.0) => None,
                _ => Some(calculate_formulas("freq", value.freq.unwrap(), &tel, is_ieee754_fp)),
            },
            demanda: match value.demanda {
                None => None,
                Some(-1.0) => None,
                _ => Some(calculate_formulas("demanda", value.demanda.unwrap(), &tel, is_ieee754_fp)),
            },
            demanda_at: match value.demanda_at {
                None => None,
                Some(-1.0) => None,
                _ => Some(calculate_formulas("demanda_at", value.demanda_at.unwrap(), &tel, is_ieee754_fp)),
            },
            demanda_ap: match value.demanda_ap {
                None => None,
                Some(-1.0) => None,
                _ => Some(calculate_formulas("demanda_ap", value.demanda_ap.unwrap(), &tel, is_ieee754_fp)),
            },
            demanda_med_at: match value.demanda_med_at {
                None => None,
                Some(-1.0) => None,
                _ => Some(calculate_formulas("demanda_med_at", value.demanda_med_at.unwrap(), &tel, is_ieee754_fp))
            },
            erro: match value.erro {
                None => None,
                Some(-1.0) => None,
                _ => Some(calculate_formulas("erro", value.erro.unwrap(), &tel, false)),
            },
        };
        Ok(result)
    }
}
