use std::str::FromStr;
use std::sync::Arc;
use std::clone::Clone;
use actix_web::{post, web, HttpResponse, Responder};
use chrono::NaiveDate;
use serde_json::json;
use crate::db::entities::energy_demand_minutes_hist::{get_demand_energy_grouped_by_hour, get_demand_energy_grouped_by_minutes, get_demand_info_by_hour, get_demand_info_by_minutes};
use crate::http::structs::energy_demand::{GetDemandInfoResponse, GetEnergyDemandResponse, ReqParamsGetDemandEnergy};
use crate::schedules::scheduler::write_to_log_file_thread;
use crate::GlobalVars;

pub fn energy_demand_config_routes() -> actix_web::Scope {
    web::scope("/energy_demand")
    .service(get_energy_demand_by_unit)
}

#[post("/get-energy-demand-by-unit")]
async fn get_energy_demand_by_unit(req_body: web::Json<ReqParamsGetDemandEnergy>, globs: web::Data<Arc<GlobalVars>>) -> impl Responder {
    let start_date_formatted = format!("{} 00:00:00", req_body.start_date);
    let end_date_formatted = format!("{} 23:59:59", req_body.end_date);
    let mut response_demand_hist: Vec<GetEnergyDemandResponse> = Vec::new();
    let mut response_demand_info: Option<GetDemandInfoResponse> = None;

    if req_body.electric_circuits_ids.is_empty() {
        let msg_error = format!("Erro ao obter dados de histórico de demanda, parâmetros incorretos, {:?}", req_body);
        write_to_log_file_thread(&msg_error, 0, "ERROR");
        println!("{}", msg_error);
        return HttpResponse::InternalServerError().body(msg_error)
    }

    if req_body.hour_graphic {
        response_demand_hist = match get_demand_energy_grouped_by_hour(req_body.unit_id, req_body.electric_circuits_ids.clone(), &start_date_formatted, &end_date_formatted, &globs) {
            Ok(res) => res,
            Err(err) => {
                let msg_error = format!("Erro ao obter dados do histórico de demanda agrupados por hora 1: {}", err);
                write_to_log_file_thread(&msg_error, 0, "ERROR");
                println!("{}", msg_error);
                return HttpResponse::InternalServerError().body(msg_error)
            }
        };

        response_demand_info = match get_demand_info_by_hour(req_body.unit_id, req_body.electric_circuits_ids.clone(), &start_date_formatted, &end_date_formatted, &globs) {
            Ok(res) => res,
            Err(err) => {
                let msg_error = format!("Erro ao obter dados do histórico de demanda agrupados por hora 2: {}", err);
                write_to_log_file_thread(&msg_error, 0, "ERROR");
                println!("{}", msg_error);
                return HttpResponse::InternalServerError().body(msg_error)
            }
        };
    } else {
        response_demand_hist = match get_demand_energy_grouped_by_minutes(req_body.unit_id, req_body.electric_circuits_ids.clone(), &start_date_formatted, &end_date_formatted, &globs) {
            Ok(res) => res,
            Err(err) => {
                let msg_error = format!("Erro ao obter dados do histórico de demanda agrupados por minutos 1: {}", err);
                write_to_log_file_thread(&msg_error, 0, "ERROR");
                println!("{}", msg_error);
                return HttpResponse::InternalServerError().body(msg_error)
            }
        };

        response_demand_info = match get_demand_info_by_minutes(req_body.unit_id, req_body.electric_circuits_ids.clone(), &start_date_formatted, &end_date_formatted, &globs) {
            Ok(res) => res,
            Err(err) => {
                let msg_error = format!("Erro ao obter dados do histórico de demanda agrupados por minutos 2: {}", err);
                write_to_log_file_thread(&msg_error, 0, "ERROR");
                println!("{}", msg_error);
                return HttpResponse::InternalServerError().body(msg_error)
            }
        };
    }

    HttpResponse::Ok().json(json!({
        "demands": response_demand_hist,
        "demand_info": response_demand_info,
    }))
}
