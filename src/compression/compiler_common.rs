// Esse lint é permitido por causa da comparação correta em [createCompiledVar]
#![allow(clippy::float_cmp)]

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SingleVariableCompiler {
    pub vec: Vec<(String, isize)>,
    pub value_count: isize,
    pub last_index: isize,
    pub last_value: String,
    pub saved_length: isize,
    pub has_error: bool,
    /// Comprimento mínimo (em amostras) que uma variável tem que manter em um valor para ser compilada. Caso esse número de amostras não seja atingido,
    /// considera-se que a variável nunca mudou.
    /// Default: 1 (sempre registra variações).
    min_run_length: isize,
}

impl SingleVariableCompiler {
    pub fn create() -> SingleVariableCompiler {
        let mut rtcVar = SingleVariableCompiler {
            vec: Vec::new(),
            value_count: 0,
            last_index: -1,
            last_value: String::new(),
            saved_length: 0,
            has_error: false,
            min_run_length: 1,
        };
        rtcVar.clear_compiled_var();
        return rtcVar;
    }

    pub fn clear_compiled_var(&mut self) {
        self.vec.clear();
        self.value_count = 0;
        self.last_index = -1;
        self.last_value.clear();
        self.saved_length = 0;
        self.has_error = false;
    }

    pub fn adc_ponto(&mut self, index: isize, valor: &str, tolerance_time: isize) {
        if self.has_error {
            return;
        }

        // tolerância tem que ser no mínimo do tamanho do meu filtro de tamanho de run
        let tolerance_time = self.min_run_length.max(tolerance_time);

        if (index < 0) || (index < self.last_index) {
            println!("Warn 63: {} {}", index, self.last_index);
            return;
            // self.clear_compiled_var();
            // self.has_error = true;
            // return true;
        }
        if index == self.last_index {
            if self.value_count > 0 {
                self.value_count -= 1;
                self.last_index -= 1;
            } else {
                println!("Warn 70: {} {} {}", index, self.last_index, self.value_count);
            }
        }
        let delta = index - self.last_index;
        if (delta >= tolerance_time) && (!self.last_value.is_empty()) {
            // println!("Houve delta grande: " + delta + " no index " + index + " com valor: '" + valor + "' e o último era '" + self.last_value + "'" + endl)
            self.salvar_trecho();
            self.last_value.clear();
            // [0123456789]
            // [111       ]
            // [111   2   ]
            self.value_count = index - self.saved_length;
            self.last_index = index - 1;
        }
        let delta = index - self.last_index;
        if valor == self.last_value {
            // sameValue
            self.last_index = index;
            self.value_count += delta;
        } else {
            if delta > 0 {
                self.last_index = index - 1;
                self.value_count += (delta - 1);
            }
            // println!("Houve troca de valor no index " + index + " com valor: '" + valor + "' e o último era '" + self.last_value + "'" + endl)
            self.salvar_trecho();
            self.last_value.clear();
            self.last_value.push_str(valor);
            self.last_index = index;
            self.value_count = 1;
        }
    }

    fn salvar_trecho(&mut self) {
        // , valorTrecho: &str, indexBeginTrecho: isize, indexEndTrecho: isize
        if self.value_count == 0 {
            return;
        }

        // Filtramos instâncias de mudanças de valores mais curtas que algum comprimento dado
        // backtrack para última entrada salva e considera como se esses pontos fossem do valor anterior.
        if self.value_count < self.min_run_length && !self.vec.is_empty() {
            // self.vec definitivamente não está vazio portanto `Vec::last_mut` sempre retorna Some.
            let last_entry = self.vec.last_mut().unwrap();
            last_entry.1 += self.value_count;
            self.saved_length += self.value_count;
            self.value_count = 0;

            return;
        }

        self.vec.push((std::mem::take(&mut self.last_value), self.value_count));
        self.saved_length += self.value_count;
        self.value_count = 0;
    }

    pub fn completar_periodo(&mut self, max_points: isize) {
        if self.has_error {
            return;
        }
        if self.last_index >= max_points {
            println!("Error 228, {} {} {}", self.has_error, self.last_index, max_points);
            self.clear_compiled_var();
            self.has_error = true;
            return;
        }
        if self.last_index < 0 {
            // println!("Error 103, {} {} {}", self.has_error, self.last_index, max_points);
            return;
        }
        if self.last_index < (max_points - 1) {
            self.adc_ponto(max_points - 1, "", 1);
        }
    }

    pub fn obter_vetor_completo(&mut self) -> String {
        if self.has_error {
            println!("Error 101, {} {}", self.has_error, self.last_index);
            return "".to_owned();
        }
        if (self.last_index < 0) {
            // println!("Error 103, {} {} {}", self.has_error, self.last_index, max_points);
            return "".to_owned();
        }
        self.salvar_trecho();
        let v = std::mem::take(&mut self.vec);
        v.into_iter()
            .filter_map(|(val, count)| {
                match count {
                    0isize => None,
                    1isize => Some(val),
                    c => Some(format!("{val}*{c}")),
                }
            })
            .collect::<Vec<_>>()
            .join(",")
    }

    pub fn fechar_vetor_completo(&mut self, max_points: isize) -> String {
        self.completar_periodo(max_points);
        return self.obter_vetor_completo();
    }

    pub fn had_error(&self) -> bool {
        self.has_error
    }

    pub fn is_empty(&self) -> bool {
        self.last_index < 0
    }
}

#[derive(Debug)]
pub struct SingleVariableCompilerBuilder {
    min_run_length: isize,
    round_steps_num: i32,
    round_steps_den: i32,
    filt_fac_new: f64,
}

impl SingleVariableCompilerBuilder {
    pub fn new() -> Self {
        Self {
            min_run_length: 1,
            round_steps_num: 1,
            round_steps_den: 1,
            filt_fac_new: 0.5,
        }
    }

    pub fn with_min_run_length(mut self, l: isize) -> Self {
        self.min_run_length = l;
        self
    }

    pub fn with_round_steps(mut self, num: i32, den: i32) -> Self {
        self.round_steps_num = num;
        self.round_steps_den = den;
        self
    }

    pub fn with_filter_factor(mut self, f: f64) -> Self {
        self.filt_fac_new = f;
        self
    }

    pub fn build_common(self) -> SingleVariableCompiler {
        let mut c = SingleVariableCompiler::create();
        c.min_run_length = self.min_run_length;
        c
    }

    pub fn build_float(self) -> SingleVariableCompilerFloat {
        let mut c = SingleVariableCompilerFloat::create(self.round_steps_num, self.round_steps_den, self.filt_fac_new);
        c.inner.min_run_length = self.min_run_length;
        c
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct SingleVariableCompilerFloat {
    pub inner: SingleVariableCompiler,
    pub last_float_value: Option<f64>,
    pub filtered_value: f64,
    pub filt_fac_last: f64,
    pub filt_fac_new: f64,
    pub round_steps: f64,
}

impl SingleVariableCompilerFloat {
    pub fn create(round_steps_num: i32, round_steps_den: i32, filt_fac_new: f64) -> SingleVariableCompilerFloat {
        let mut rtcVar = SingleVariableCompilerFloat {
            inner: SingleVariableCompiler::create(),
            round_steps: (f64::from(round_steps_num) / f64::from(round_steps_den)), // 2
            filt_fac_new, // 0.6
            filt_fac_last: (1.0 - filt_fac_new),
            last_float_value: Some(0.0),
            filtered_value: 0.0,
        };
        rtcVar.clear_compiled_var();
        return rtcVar;
    }

    pub fn clear_compiled_var(&mut self) {
        self.last_float_value = Some(0.0);
        self.filtered_value = 0.0;
        self.inner.clear_compiled_var();
    }

    pub fn adc_ponto_float(&mut self, index: isize, fval: Option<f64>, tolerance_time: isize) {
        if self.inner.has_error {
            return;
        }
        let fval = match fval {
            Some(v) => v,
            None => {
                self.last_float_value = None;
                return self.inner.adc_ponto(index, "", tolerance_time);
            }
        };

        let delta = index - self.inner.last_index;
        if (self.inner.saved_length == 0) || self.inner.last_value.is_empty() || (delta > tolerance_time) {
            self.filtered_value = fval;
        }

        let etapa1 = fval * self.filt_fac_new + self.filtered_value * self.filt_fac_last;
        let etapa2 = (etapa1 * self.round_steps).round() / self.round_steps;
        self.filtered_value = etapa1;
        if (self.inner.saved_length > 0) && (self.last_float_value == Some(etapa2)) {
            // sameValue
            return self.inner.adc_ponto(index, &self.inner.last_value.to_owned(), tolerance_time);
        }
        self.last_float_value = Some(etapa2);

        let sval = etapa2.to_string();

        return self.inner.adc_ponto(index, &sval, tolerance_time);
    }

    pub fn completar_periodo(&mut self, max_points: isize) {
        self.inner.completar_periodo(max_points);
    }

    pub fn obter_vetor_completo(&mut self) -> String {
        self.inner.obter_vetor_completo()
    }

    pub fn fechar_vetor_completo(&mut self, max_points: isize) -> String {
        self.inner.fechar_vetor_completo(max_points)
    }

    pub fn had_error(&self) -> bool {
        self.inner.had_error()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}
