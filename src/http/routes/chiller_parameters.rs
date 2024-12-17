use std::sync::Arc;

use actix_web::{post, web, HttpResponse, Responder};
use serde_json::json;

use crate::{db::entities::chiller::{chiller_hx_parameters_minutes_hist::{get_chiller_hx_parameters_hist_hour, get_chiller_hx_parameters_hist_minutes}, chiller_parameters_changes_hist::get_chiller_parameters_changes_hist, chiller_xa_hvar_parameters_minutes_hist::{get_chiller_xa_hvar_parameters_hist_hour, get_chiller_xa_hvar_parameters_hist_minutes}, chiller_xa_parameters_minutes_hist::{get_chiller_xa_parameters_hist_hour, get_chiller_xa_parameters_hist_minutes}}, http::structs::chiller_parameters::ReqParamsGetChillerParametersHist, schedules::scheduler::write_to_log_file_thread, GlobalVars};

pub fn chiller_parameters_routes() -> actix_web::Scope {
    web::scope("/chiller_parameters")
    .service(get_chiller_hx_parameters_hist)
    .service(get_chiller_xa_parameters_hist)
    .service(get_chiller_xa_hvar_parameters_hist)
}

#[post("/get-chiller-hx-parameters-hist")]
async fn get_chiller_hx_parameters_hist(req_body: web::Json<ReqParamsGetChillerParametersHist>, globs: web::Data<Arc<GlobalVars>>) -> impl Responder {
    let start_date_formatted = format!("{} 00:00:00", req_body.start_date);
    let end_date_formatted = format!("{} 23:59:59", req_body.end_date);


    let response_parameters_grouped_hist = if req_body.hour_graphic {
        match get_chiller_hx_parameters_hist_hour(&req_body.device_code, &start_date_formatted, &end_date_formatted, &globs) {
            Ok(res) => res,
            Err(err) => {
                let msg_error = format!("Erro ao obter dados do histórico de parâmetros do chiller agrupados por hora: {}", err);
                write_to_log_file_thread(&msg_error, 0, "ERROR");
                println!("{}", msg_error);
                return HttpResponse::InternalServerError().body(msg_error)
            }
        }
    } else {
        match get_chiller_hx_parameters_hist_minutes(&req_body.device_code, &start_date_formatted, &end_date_formatted, &globs) {
            Ok(res) => res,
            Err(err) => {
                let msg_error = format!("Erro ao obter dados do histórico de parâmetros do chiller agrupados por 10 minutos: {}", err);
                write_to_log_file_thread(&msg_error, 0, "ERROR");
                println!("{}", msg_error);
                return HttpResponse::InternalServerError().body(msg_error)
            }
        }
    };
    
    let response_parameters_changes_hist = match get_chiller_parameters_changes_hist(&req_body.device_code, &start_date_formatted, &end_date_formatted, &globs) {
        Ok(res) => res,
        Err(err) => {
            let msg_error = format!("Erro ao obter dados do histórico de parâmetros do chiller: {}", err);
            write_to_log_file_thread(&msg_error, 0, "ERROR");
            println!("{}", msg_error);
            return HttpResponse::InternalServerError().body(msg_error)
        }
    };
    
    HttpResponse::Ok().json(json!({
        "parameters_grouped_hist": response_parameters_grouped_hist,
        "parameters_changes_hist": response_parameters_changes_hist
    }))
}

#[post("/get-chiller-xa-parameters-hist")]
async fn get_chiller_xa_parameters_hist(req_body: web::Json<ReqParamsGetChillerParametersHist>, globs: web::Data<Arc<GlobalVars>>) -> impl Responder {
    let start_date_formatted = format!("{} 00:00:00", req_body.start_date);
    let end_date_formatted = format!("{} 23:59:59", req_body.end_date);


    let response_parameters_grouped_hist = if req_body.hour_graphic {
        match get_chiller_xa_parameters_hist_hour(&req_body.device_code, &start_date_formatted, &end_date_formatted, &globs) {
            Ok(res) => res,
            Err(err) => {
                let msg_error = format!("Erro ao obter dados do histórico de parâmetros do chiller xa agrupados por hora: {}", err);
                write_to_log_file_thread(&msg_error, 0, "ERROR");
                println!("{}", msg_error);
                return HttpResponse::InternalServerError().body(msg_error)
            }
        }
    } else {
        match get_chiller_xa_parameters_hist_minutes(&req_body.device_code, &start_date_formatted, &end_date_formatted, &globs) {
            Ok(res) => res,
            Err(err) => {
                let msg_error = format!("Erro ao obter dados do histórico de parâmetros do chiller xa agrupados por 10 minutos: {}", err);
                write_to_log_file_thread(&msg_error, 0, "ERROR");
                println!("{}", msg_error);
                return HttpResponse::InternalServerError().body(msg_error)
            }
        }
    };
    
    let response_parameters_changes_hist = match get_chiller_parameters_changes_hist(&req_body.device_code, &start_date_formatted, &end_date_formatted, &globs) {
        Ok(res) => res,
        Err(err) => {
            let msg_error = format!("Erro ao obter dados do histórico de parâmetros do chiller xa: {}", err);
            write_to_log_file_thread(&msg_error, 0, "ERROR");
            println!("{}", msg_error);
            return HttpResponse::InternalServerError().body(msg_error)
        }
    };
    
    HttpResponse::Ok().json(json!({
        "parameters_grouped_hist": response_parameters_grouped_hist,
        "parameters_changes_hist": response_parameters_changes_hist
    }))
}

#[post("/get-chiller-xa-hvar-parameters-hist")]
async fn get_chiller_xa_hvar_parameters_hist(req_body: web::Json<ReqParamsGetChillerParametersHist>, globs: web::Data<Arc<GlobalVars>>) -> impl Responder {
    let start_date_formatted = format!("{} 00:00:00", req_body.start_date);
    let end_date_formatted = format!("{} 23:59:59", req_body.end_date);


    let response_parameters_grouped_hist = if req_body.hour_graphic {
        match get_chiller_xa_hvar_parameters_hist_hour(&req_body.device_code, &start_date_formatted, &end_date_formatted, &globs) {
            Ok(res) => res,
            Err(err) => {
                let msg_error = format!("Error when obtaining chiller parameter history data xa_hvar grouped by hour: {}", err);
                write_to_log_file_thread(&msg_error, 0, "ERROR");
                println!("{}", msg_error);
                return HttpResponse::InternalServerError().body(msg_error)
            }
        }
    } else {
        match get_chiller_xa_hvar_parameters_hist_minutes(&req_body.device_code, &start_date_formatted, &end_date_formatted, &globs) {
            Ok(res) => res,
            Err(err) => {
                let msg_error = format!("Error when obtaining chiller parameter history data xa_hvar grouped by 10 minutes: {}", err);
                write_to_log_file_thread(&msg_error, 0, "ERROR");
                println!("{}", msg_error);
                return HttpResponse::InternalServerError().body(msg_error)
            }
        }
    };
    
    let response_parameters_changes_hist = match get_chiller_parameters_changes_hist(&req_body.device_code, &start_date_formatted, &end_date_formatted, &globs) {
        Ok(res) => res,
        Err(err) => {
            let msg_error = format!("Error retrieving data from chiller parameter history xa_hvar: {}", err);
            write_to_log_file_thread(&msg_error, 0, "ERROR");
            println!("{}", msg_error);
            return HttpResponse::InternalServerError().body(msg_error)
        }
    };
    
    HttpResponse::Ok().json(json!({
        "parameters_grouped_hist": response_parameters_grouped_hist,
        "parameters_changes_hist": response_parameters_changes_hist
    }))
}
