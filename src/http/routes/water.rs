use std::sync::Arc;
use std::clone::Clone;
use actix_web::{post, web, HttpResponse, Responder};
use serde_json::json;
use crate::db::entities::water_consumption_forecast::get_forecast_usage;
use crate::db::entities::waters_hist::{get_day_usage_history, get_year_usage_history, get_hour_usage_history, get_water_info_by_day_graphic, get_water_info_by_hour_graphic};
use crate::http::structs::water::{GetWaterConsumption, GetWaterDayGraphicInfoResponse, GetWaterForecastUsageRequestBody, GetWaterGraphicInfoResponse, GetWaterUsageHistoryRequest, GetWaterUsageHistoryResponse};
use crate::schedules::scheduler::write_to_log_file_thread;
use crate::{db::entities::waters_hist::{get_water_month_usage, get_water_year_usage, get_water_dates_year_usage}, http::structs::water::{GetWaterUsageRequestBody, GetWaterYearUsageRequestBody}, GlobalVars};

pub fn water_config_routes() -> actix_web::Scope {
    web::scope("/water")
    .service(water_month_usage)
    .service(water_dates_year_usage)
    .service(water_year_usage)
    .service(water_usage_history)
    .service(water_forecast_usage)
}

#[post("/get-month-usage")]
async fn water_month_usage(req_body: web::Json<GetWaterUsageRequestBody>, globs: web::Data<Arc<GlobalVars>>) -> impl Responder {
    
    let params = GetWaterUsageRequestBody {
        unitIds: req_body.unitIds.clone(),
        startDate: req_body.startDate.to_string(),
        endDate: req_body.endDate.to_string(),
    };

    match get_water_month_usage(params, &globs){
        Ok(response) => return HttpResponse::Ok().json(response),
        Err(error) => return HttpResponse::InternalServerError().body(format!("Erro ao obter o consumo por mes de água: {}", error))
    };
}

#[post("/get-year-usage")]
async fn water_year_usage(req_body: web::Json<GetWaterYearUsageRequestBody>, globs: web::Data<Arc<GlobalVars>>) -> impl Responder {
    
    let params = GetWaterYearUsageRequestBody {
        unitIds: req_body.unitIds.clone(),
        startDate: req_body.startDate.to_string(),
        endDate: req_body.endDate.to_string(),
    };

    match get_water_year_usage(params, &globs){
        Ok(response) => return HttpResponse::Ok().json(response),
        Err(error) => return HttpResponse::InternalServerError().body(format!("Erro ao obter o consumo total de água: {}", error))
    };
}

#[post("/get-dates-year-usage")]
async fn water_dates_year_usage(req_body: web::Json<GetWaterUsageRequestBody>, globs: web::Data<Arc<GlobalVars>>) -> impl Responder {  
    let params = GetWaterUsageRequestBody {
        unitIds: req_body.unitIds.clone(),
        startDate: req_body.startDate.to_string(),
        endDate: req_body.endDate.to_string(),
    };

    match get_water_dates_year_usage(params, &globs){
        Ok(response) => return HttpResponse::Ok().json(response),
        Err(error) => {
            let msg_error = format!("Erro ao obter a analise de água na rota /get-dates-year-usage: {}", error);
            write_to_log_file_thread(&msg_error, 0, "ERROR");
            println!("{}", msg_error);
            return HttpResponse::InternalServerError().body(format!("Erro ao obter a analise de água: {}", error))
        }
    };
}

#[post("/get-usage-history")]
async fn water_usage_history(req_body: web::Json<GetWaterUsageHistoryRequest>, globs: web::Data<Arc<GlobalVars>>) -> impl Responder {
    let start_date_formatted = format!("{} 00:00:00", req_body.start_date);
    let end_date_formatted = format!("{} 23:59:59", req_body.end_date);
    let last_start_date_formatted = format!("{} 00:00:00", req_body.last_start_date);
    let last_end_date_formatted = format!("{} 23:59:59", req_body.last_end_date);
    
    let mut response_water_hist: Vec<GetWaterUsageHistoryResponse> = Vec::new();
    let mut response_day_water_info: Option<GetWaterDayGraphicInfoResponse> = None;
    let mut response_day_water_info_last: Option<GetWaterDayGraphicInfoResponse> = None;
    let mut response_hour_water_info: Option<GetWaterConsumption> = None;
    let mut response_hour_water_info_last: Option<GetWaterConsumption> = None;
    let mut response_year_water_info: Option<GetWaterDayGraphicInfoResponse> = None;
    let mut response_year_water_info_last: Option<GetWaterDayGraphicInfoResponse> = None;

    if req_body.hour_graphic {
        response_water_hist = match get_hour_usage_history(req_body.unit_id, &start_date_formatted, &end_date_formatted, &globs) {
            Ok(response) => response,
            Err(error) => {
                let msg_error = format!("Error retrieving hourly water history on route: /get-usage-history, {}", error);
                write_to_log_file_thread(&msg_error, 0, "ERROR");
                println!("{}", msg_error);
                return HttpResponse::InternalServerError().body(format!("Error obtaining water history: {}", error))
            }
        };

        response_hour_water_info = match get_water_info_by_hour_graphic(req_body.unit_id, &start_date_formatted, &end_date_formatted, &globs) {
            Ok(response) => response,
            Err(error) => {
                let msg_error = format!("Error retrieving water history information per hour on the route: /get-usage-history, {}", error);
                write_to_log_file_thread(&msg_error, 0, "ERROR");
                println!("{}", msg_error);
                return HttpResponse::InternalServerError().body(format!("Error obtaining water history: {}", error))
            }
        };

        response_hour_water_info_last = match get_water_info_by_hour_graphic(req_body.unit_id, &last_start_date_formatted, &last_end_date_formatted, &globs) {
            Ok(response) => response,
            Err(error) => {
                let msg_error = format!("Error retrieving last water history information per hour on the route: /get-usage-history, {}", error);
                write_to_log_file_thread(&msg_error, 0, "ERROR");
                println!("{}", msg_error);
                return HttpResponse::InternalServerError().body(format!("Error obtaining water history: {}", error))
            }
        };

        HttpResponse::Ok().json(json!({
            "consumption_hist": response_water_hist,
            "consumption_info": response_hour_water_info,
            "consumption_info_last": response_hour_water_info_last,
        }))
    } 

    else if req_body.year_graphic {
        response_water_hist = match get_year_usage_history(req_body.unit_id, &start_date_formatted, &end_date_formatted, &globs) {
            Ok(response) => response,
            Err(error) => {
                let msg_error = format!("Error retrieving yearly water history on route: /get-usage-history, {}", error);
                write_to_log_file_thread(&msg_error, 0, "ERROR");
                println!("{}", msg_error);
                return HttpResponse::InternalServerError().body(format!("Error obtaining water history: {}", error))
            }
        };

        response_year_water_info = match get_water_info_by_day_graphic(req_body.unit_id, &start_date_formatted, &end_date_formatted, &globs) {
            Ok(response) => response,
            Err(error) => {
                let msg_error = format!("Error retrieving water history information per year on the route: /get-usage-history, {}", error);
                write_to_log_file_thread(&msg_error, 0, "ERROR");
                println!("{}", msg_error);
                return HttpResponse::InternalServerError().body(format!("Error obtaining water history: {}", error))
            }
        };

        response_year_water_info_last = match get_water_info_by_day_graphic(req_body.unit_id, &last_start_date_formatted, &last_end_date_formatted, &globs) {
            Ok(response) => response,
            Err(error) => {
                let msg_error = format!("Error retrieving last water history information per year on the route: /get-usage-history, {}", error);
                write_to_log_file_thread(&msg_error, 0, "ERROR");
                println!("{}", msg_error);
                return HttpResponse::InternalServerError().body(format!("Error obtaining water history: {}", error))
            }
        };

        HttpResponse::Ok().json(json!({
            "consumption_hist": response_water_hist,
            "consumption_info": response_year_water_info,
            "consumption_info_last": response_year_water_info_last,
        }))
    }
    
    else {
        response_water_hist = match get_day_usage_history(req_body.unit_id, &start_date_formatted, &end_date_formatted, &globs) {
            Ok(response) => response,
            Err(error) => {
                let msg_error = format!("Error retrieving per day water history on route: /get-usage-history, {}", error);
                write_to_log_file_thread(&msg_error, 0, "ERROR");
                println!("{}", msg_error);
                return HttpResponse::InternalServerError().body(format!("Error obtaining water history: {}", error))
            }
        };

        response_day_water_info = match get_water_info_by_day_graphic(req_body.unit_id, &start_date_formatted, &end_date_formatted, &globs) {
            Ok(response) => response,
            Err(error) => {
                let msg_error = format!("Error retrieving water history information per day on the route: /get-usage-history, {}", error);
                write_to_log_file_thread(&msg_error, 0, "ERROR");
                println!("{}", msg_error);
                return HttpResponse::InternalServerError().body(format!("Error obtaining water history: {}", error))
            }
        };

        response_day_water_info_last = match get_water_info_by_day_graphic(req_body.unit_id, &last_start_date_formatted, &last_end_date_formatted, &globs) {
            Ok(response) => response,
            Err(error) => {
                let msg_error = format!("Error retrieving last water history information per day on the route: /get-usage-history, {}", error);
                write_to_log_file_thread(&msg_error, 0, "ERROR");
                println!("{}", msg_error);
                return HttpResponse::InternalServerError().body(format!("Error obtaining water history: {}", error))
            }
        };

        HttpResponse::Ok().json(json!({
            "consumption_hist": response_water_hist,
            "consumption_info": response_day_water_info,
            "consumption_info_last": response_day_water_info_last,
        }))
    }
}

#[post("/get-forecast-usage")]
async fn water_forecast_usage(req_body: web::Json<GetWaterForecastUsageRequestBody>, globs: web::Data<Arc<GlobalVars>>) -> impl Responder {
    
    match get_forecast_usage(req_body.unit_id, &req_body.forecast_date, &globs){
        Ok(response) => return HttpResponse::Ok().json(response),
        Err(error) => {
            let msg_error = format!("Error when forecasting water consumption on route /get-forecast-usage: {}", error);
            write_to_log_file_thread(&msg_error, 0, "ERROR");
            println!("{}", msg_error);

            return HttpResponse::InternalServerError().body(format!("Error when forecasting water consumption: {}", error))
        }
    };
}
