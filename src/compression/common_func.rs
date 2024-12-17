use std::str::FromStr;
use std::fmt::Write;
use crate::models::external_models::device::MachineAutomInterval;

pub fn calcular_tempo_online(vec: &str) -> (f64) {
    let mut hours_online = 0.0;

    if vec.is_empty() {
        return hours_online;
    }
    let imax = usize::try_from(vec.len() - 1).unwrap();
    let mut i: usize = 0;
    let mut ival: i64 = -1;
    let mut iast: i64 = -1;
    let mut value;
    loop {
        if ival < 0 {
            ival = i as i64;
        }
        if i > imax || vec.as_bytes()[i] == b',' {
            let duration: isize;
            if iast < 0 {
                value = &vec[(ival as usize)..i];
                duration = 1;
            } else {
                value = &vec[(ival as usize)..(iast as usize)];
                duration = isize::from_str(&vec[((iast + 1) as usize)..i]).unwrap();
            }
            if !value.is_empty() {
                hours_online += (duration as f64) / 3600.0;
            }
            ival = -1;
            iast = -1;
        } else if vec.as_bytes()[i] == b'*' {
            iast = i as i64;
        }
        if i > imax {
            break;
        }
        i += 1;
    }
    return hours_online;
}

pub fn check_amount_minutes_offline(minutes_to_check: i32, vec: Vec<String>, start_date: &str) -> f64 {
    let start_timestamp = chrono::NaiveDateTime::parse_from_str(start_date, "%Y-%m-%dT%H:%M:%S").unwrap();

    if vec.is_empty() {
        return 0.0;
    }

    let minutes_verified = if minutes_to_check == 0 { 15 } else { minutes_to_check };

    let mut previous_timestamp = start_timestamp;
    let mut offline_periods = 0;

    for ts_str in vec {
        let current_timestamp = chrono::NaiveDateTime::parse_from_str(&ts_str, "%Y-%m-%dT%H:%M:%S").unwrap();
        if current_timestamp.date() < start_timestamp.date() { continue; }
        if current_timestamp.date() > previous_timestamp.date() { break; }

        let diff_in_minutes = (current_timestamp - previous_timestamp).num_minutes();

        if diff_in_minutes > minutes_verified.into() {
            offline_periods += (diff_in_minutes / minutes_verified as i64) as i32;
        }

        previous_timestamp = current_timestamp;
    }

    let end_of_day_timestamp = previous_timestamp.date().and_hms(23, 59, 59);
    let final_diff_in_minutes = (end_of_day_timestamp - previous_timestamp).num_minutes();

    if final_diff_in_minutes > minutes_verified.into() {
        offline_periods += (final_diff_in_minutes / minutes_verified as i64) as i32;
    }

    let total_intervals_in_day = 1440 / minutes_verified;

    let online_percentage = ((total_intervals_in_day - offline_periods) as f64 / total_intervals_in_day as f64) * 100.0;
    if online_percentage < 0.0 { 0.0 } else { online_percentage }
}

#[derive(Debug, Clone)]
struct CompiledHistoryVar {
    c: Vec<i32>,
    v: Vec<Option<i32>>,
}

#[derive(Debug, Clone)]
struct Lcmp {
    v: Vec<Option<i32>>,
    c: Vec<i32>,
}

fn merge_events_list(var_list: &mut Vec<CompiledHistoryVar>) -> (Vec<i32>, Vec<Vec<Option<i32>>>) {
    // Garantir que não tenha nenhum elemento null no var_list
    for var in var_list.iter_mut() {
        if var.v.is_empty() {
            var.c = vec![];
            var.v = vec![];
        }
    }

    let mut dur = vec![]; // vetor c final, equivalente ao eixo x comum a todas as variáveis
    let mut vals = vec![vec![]; var_list.len()]; // cada índice contém o vetor de valores da variável
    let mut p = vec![0; var_list.len()]; // índice no vetor v (e c) de cada variável
    let mut v: Vec<Option<i32>> = vec![]; // valor de cada variável no ponto atual do algoritmo
    let mut c = vec![]; // quantos segundos cada variável vai permanecer com o valor atual

    for var in var_list.iter() {
        if !var.v.is_empty() {
            v.push(var.v[0]);
            c.push(var.c[0]);
        } else {
            v.push(None);
            c.push(0);
        }
    }

    while c.iter().any(|&x| x != 0) {
        let step = *c.iter().filter(|&&x| x != 0).min().unwrap(); // quantos segundos até alguma variável mudar de valor
        dur.push(step); // step = quantos segundos todas as variáveis vão permanecer com os mesmos valores
        for i in 0..var_list.len() {
            vals[i].push(v[i].clone()); // coloca o valor atual de cada variável no vetor vals
            if c[i] != 0 {
                c[i] -= step; // desconta do vetor c a quantidade de segundos já contabilizada
            }
            if c[i] == 0 {
                if !var_list[i].v.is_empty() && p[i] < var_list[i].v.len() - 1 {
                    p[i] += 1;
                    v[i] = var_list[i].v[p[i]].clone();
                    c[i] = var_list[i].c[p[i]];
                } else {
                    v[i] = None;
                }
            }
        }
    }

    (dur, vals)
}

pub fn consumption_by_hour(lcmp: &String) -> Vec<i32> {
    let lcmp_aux = parse_lcmp(lcmp);
    let n_horas = CompiledHistoryVar {
        v: (0..24).map(|i| Some(i)).collect(),
        c: vec![3600; 24],
    };

    let mut var_list = vec![
        CompiledHistoryVar {
            v: lcmp_aux.v,
            c: lcmp_aux.c,
        },
        n_horas,
    ];

    let list = merge_events_list(&mut var_list);

    let mut cons = vec![0; 24];
    for i in 0..list.0.len() {
        if let Some(l1) = list.1[0][i] {
            if l1 != 1 {
                continue;
            }
        } else {
            continue;
        }

        if let Some(n_hora) = list.1[1][i] {
            cons[n_hora as usize] += list.0[i];
        }
    }

    cons
}

fn parse_lcmp(input: &str) -> Lcmp {
    let mut v: Vec<Option<i32>> = Vec::new();
    let mut c: Vec<i32> = Vec::new();
    
    for pair in input.split(",") {
        let parts: Vec<&str> = pair.split('*').collect();
        if parts.len() == 2 {
            if parts[0].is_empty() {
                v.push(None);
            } else {
                if let Ok(value) = parts[0].parse::<i32>() {
                    v.push(Some(value));
                } else {
                    v.push(None);
                }
            }
            if let Ok(seconds) = parts[1].parse::<i32>() {
                c.push(seconds);
            }
        }
    }

    Lcmp { v, c }
}

fn is_outside_intervals(acumulado: i32, intervals: &Vec<MachineAutomInterval>) -> bool {
    for i in 0..intervals.len() {
        if intervals[i].must_be_on {
            if acumulado >= intervals[i].seconds_start && acumulado <= intervals[i].seconds_end {
                return false; // Dentro de um intervalo de ligar
            }
        } else {
            if acumulado >= intervals[i].seconds_start && acumulado <= intervals[i].seconds_end {
                return true; // Dentro de um intervalo de desligar
            }
            if intervals.len() == 1 {
                return false; // Dentro de um intervalo de ligar
            }
        }
    }
    true // Fora de todos os intervalos de ligar
}

pub fn calculate_l1_states(lcmp: &String, intervals: Vec<MachineAutomInterval>) -> (i32, i32, i32, i32, f64) { 
    let lcmp_parsed = parse_lcmp(lcmp);
    let mut total_on = 0;
    let mut total_off = 0;
    let mut total_on_outside_programming = 0;
    let mut acumulado_horas = 0;

    for i in 0..lcmp_parsed.v.len() {
        let valor_atual = lcmp_parsed.v[i];
        let duracao_atual = lcmp_parsed.c[i];

        let tempo_inicio = acumulado_horas;
        let tempo_fim = acumulado_horas + duracao_atual;

        acumulado_horas = tempo_fim;

        if let Some(valor_atual) = valor_atual {
            if valor_atual != 0 {
                total_on += duracao_atual;

                for t in (tempo_inicio + 1)..=tempo_fim {
                    if is_outside_intervals(t, &intervals) {
                        total_on_outside_programming += 1;
                    }
                }
            } else {
                total_off += duracao_atual;
            }
        }
    }


    let mut total_intervals_on = 0;
    let mut total_intervals_off = 0;
    for i in 0..intervals.len() {
        if intervals[i].must_be_on {
            total_intervals_on = total_intervals_on + intervals[i].seconds_end - intervals[i].seconds_start;
        } else {
            total_intervals_off = total_intervals_off + intervals[i].seconds_end - intervals[i].seconds_start
        }
    }

    let mut total_must_be_off = 0;
    if (total_intervals_off > 0 && total_intervals_on == 0) || total_intervals_on >= 86400 {
        total_must_be_off = total_intervals_off;
    } else {
        total_must_be_off = 86400 - total_intervals_on;
    };

    // Calcula o percentual de total_on_outside_programming dentro de total_must_be_off. Há tolerância de ate 5 minutos ligados fora do horário devido limitação do dispositivo.
    let percentual_outside_programming = if total_must_be_off > 0 && total_on_outside_programming > 600 {
        (total_on_outside_programming as f64 / total_must_be_off as f64) * 100.0
    } else {
        0.0
    };

    (total_on, total_off, total_on_outside_programming, total_must_be_off, percentual_outside_programming)
}

fn format_interval(interval: &MachineAutomInterval) -> String {
    let start_hours = interval.seconds_start / 3600;
    let start_minutes = (interval.seconds_start % 3600) / 60;
    let end_hours = interval.seconds_end / 3600;
    let end_minutes = (interval.seconds_end % 3600) / 60;
    let status = if interval.must_be_on { "LIGADO" } else { "DESLIGADO" };

    format!("{:02}:{:02} a {:02}:{:02} deve estar {}", start_hours, start_minutes, end_hours, end_minutes, status)
}

pub fn concatenate_intervals(intervals: Vec<MachineAutomInterval>) -> String {
    let mut result = String::new();
    for (i, interval) in intervals.iter().enumerate() {
        if i > 0 {
            result.push_str(" e ");
        }
        write!(result, "{}", format_interval(interval)).unwrap();
    }
    result
}