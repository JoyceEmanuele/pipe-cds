use std::str::FromStr;
use std::sync::Arc;
use std::clone::Clone;
use actix_web::{post, web, HttpResponse, Responder};
use chrono::NaiveDate;
use serde_json::json;
use crate::db::entities::energy_consumption_forecast::{energy_trends, get_months_with_energy_consumtion_forecast};
use crate::db::entities::energy_hist::{get_energy_analysis_list, get_energy_consumption_by_days, get_energy_consumption_by_months, get_energy_day_consumption, get_energy_hours_consumption, get_energy_units, get_months_with_energy_consumtion, get_total_units_with_constructed_area, get_total_units_with_consumption, procel_insigths};
use crate::http::structs::energy::{AnalysisHistFilterTypeEnum, AnalysisHistTypeEnum, GetEnergyAnalysisHistFilterRequestBody, GetEnergyAnalysisHistRequestBody, GetEnergyAnalysisListRequestBody, GetEnergyAnalysisListResponseComplete, GetEnergyTrendsRequestBody, GetProcelInsightsRequestBody, GetUnitListRequestBody, ReqParamsGetEnergyConsumption};
use crate::db::entities::energy_hist::{apply_energy_flags_to_unit_by_time, apply_energy_flags_to_unit_list};
use crate::schedules::energy::{adjust_consumption_by_hour, fill_consumption_by_day};
use crate::schedules::scheduler::write_to_log_file_thread;
use crate::GlobalVars;

pub fn energy_config_routes() -> actix_web::Scope {
    web::scope("/energy")
    .service(energy_analysis_list)
    .service(energy_analysis_hist)
    .service(energy_analysis_hist_filter)
    .service(units_list)
    .service(get_energy_consumption)
    .service(get_procel_insights)
    .service(get_energy_trends)
}

#[post("/get-energy-analysis-list")]
async fn energy_analysis_list(req_body: web::Json<GetEnergyAnalysisListRequestBody>, globs: web::Data<Arc<GlobalVars>>) -> impl Responder {
    
    let params = GetEnergyAnalysisListRequestBody {
        units: req_body.units.clone(),
        startDate: req_body.startDate.to_string(),
        endDate: req_body.endDate.to_string(),
        limit: req_body.limit as Option<i32>,
        offset: req_body.offset as Option<i32>,
        orderByField: req_body.orderByField.clone(),
        orderByType: req_body.orderByType.clone(),
        isDielUser: req_body.isDielUser.clone(),
        previousStartDate: req_body.previousStartDate.clone(),
        previousEndDate: req_body.previousEndDate.clone(),
        minConsumption: req_body.minConsumption.clone(),
        categoryFilter: req_body.categoryFilter.clone()
    };

    let units = match get_energy_analysis_list(&params, &globs){
        Ok(response) => response,
        Err(error) => {
            let msg_error = format!("Erro ao obter a analise de energia na rota /get-energy-analysis-list: {}", error);
            write_to_log_file_thread(&msg_error, 0, "ERROR");
            println!("{}", msg_error);
            return HttpResponse::InternalServerError().body(msg_error)
        }

    };

    let response = match apply_energy_flags_to_unit_list(units.units, req_body.isDielUser){
        Ok(response) => response,
        Err(error) => {
            let msg_error = format!("Erro ao obter a analise de energia na rota /get-energy-analysis-list: {}", error);
            write_to_log_file_thread(&msg_error, 0, "ERROR");
            println!("{}", msg_error);
            return HttpResponse::InternalServerError().body(msg_error)
        }
    };

    return HttpResponse::Ok().json(GetEnergyAnalysisListResponseComplete {
        units: response,
        classA: units.classA,
        classB: units.classB,
        classC: units.classC,
        classD: units.classD,
        classE: units.classE,
        classF: units.classF,
        classG: units.classG,
    })
}

#[post("/get-energy-analysis-hist")]
async fn energy_analysis_hist(req_body: web::Json<GetEnergyAnalysisHistRequestBody>, globs: web::Data<Arc<GlobalVars>>) -> impl Responder {
    let params = GetEnergyAnalysisHistRequestBody {
        startDate: req_body.startDate.to_string(),
        endDate: req_body.endDate.to_string(),
        filterType: req_body.filterType.clone(),
        units: req_body.units.clone(),
        isDielUser: req_body.isDielUser.clone(),
        minConsumption: req_body.minConsumption,
    };

    let dataResponse = match req_body.filterType {
        AnalysisHistTypeEnum::month => match get_energy_consumption_by_days(params.clone(), &globs){
            Ok(response) => response,
            Err(error) => {
                let msg_error = format!("Erro ao obter o consumo de energia por dias na rota /get-energy-analysis-hist: {}", error);
                write_to_log_file_thread(&msg_error, 0, "ERROR");
                println!("{}", msg_error);
                return HttpResponse::InternalServerError().body(format!("Erro ao obter o consumo de energia: {}", error))
            }
        },
        AnalysisHistTypeEnum::year => match get_energy_consumption_by_months(params.clone(), &globs){
            Ok(response) => response,
            Err(error) => {
                let msg_error = format!("Erro ao obter o consumo de energia por meses na rota /get-energy-analysis-hist: {}", error);
                write_to_log_file_thread(&msg_error, 0, "ERROR");
                println!("{}", msg_error);
                return HttpResponse::InternalServerError().body(format!("Erro ao obter o consumo de energia: {}", error))
            }
        }
    };

    let response_units_with_constructed_area = match get_total_units_with_constructed_area(params.clone(), &globs){
        Ok(response) => response,
        Err(error) => {
            let msg_error = format!("Erro ao obter o total de unidades com área construída na rota /get-energy-analysis-hist: {}", error);
            write_to_log_file_thread(&msg_error, 0, "ERROR");
            println!("{}", msg_error);
            return HttpResponse::InternalServerError().body(format!("Erro ao obter o consumo de energia: {}", error))
        }
    };

    let response_units_with_consumption = match get_total_units_with_consumption(params.clone(), &globs){
        Ok(response) => response,
        Err(error) => {
            let msg_error = format!("Erro ao obter o total de unidades com consumo na rota /get-energy-analysis-hist: {}", error);
            write_to_log_file_thread(&msg_error, 0, "ERROR");
            println!("{}", msg_error);
            return HttpResponse::InternalServerError().body(format!("Erro ao obter o consumo de energia: {}", error))
        }
    };

    match apply_energy_flags_to_unit_by_time(dataResponse, req_body.isDielUser) {
        Ok(response) => HttpResponse::Ok().json(json!({
            "units_count_with_constructed_area": response_units_with_constructed_area.units_count,
            "units_count_with_consumption": response_units_with_consumption.units_count,
            "energy_data": response
        })),
        Err(error) => {
            let msg_error = format!("Erro ao obter o consumo de energia na rota /get-energy-analysis-hist: {}", error);
            write_to_log_file_thread(&msg_error, 0, "ERROR");
            println!("{}", msg_error);
            return HttpResponse::InternalServerError().body(format!("Erro ao obter o consumo de energia: {}", error))
        }
    }
}

#[post("/get-energy-analysis-hist-filter")]
async fn energy_analysis_hist_filter(req_body: web::Json<GetEnergyAnalysisHistFilterRequestBody>, globs: web::Data<Arc<GlobalVars>>) -> impl Responder {
    
    let params = GetEnergyAnalysisHistFilterRequestBody {
        date: req_body.date.to_string(),
        filterType: req_body.filterType.clone(),
        units: req_body.units.clone()
    };

    match req_body.filterType {
        AnalysisHistFilterTypeEnum::CONSUMPTION => match get_months_with_energy_consumtion(params, &globs){
            Ok(response) => return HttpResponse::Ok().json(response),
            Err(error) => {
                let msg_error = format!("Erro ao obter os dados de consumo de energia na rota /get-energy-analysis-hist-filter: {}", error);
                write_to_log_file_thread(&msg_error, 0, "ERROR");
                println!("{}", msg_error);
                return HttpResponse::InternalServerError().body(format!("Erro ao obter os dados de consumo de energia: {}", error))
            }
        },
        AnalysisHistFilterTypeEnum::CONSUMPTION_FORECAST => match get_months_with_energy_consumtion_forecast(params, &globs){
            Ok(response) => return HttpResponse::Ok().json(response),
            Err(error) => {
                let msg_error = format!("Erro ao obter os dados de consumo de energia na rota /get-energy-analysis-hist-filter: {}", error);
                write_to_log_file_thread(&msg_error, 0, "ERROR");
                println!("{}", msg_error);
                return HttpResponse::InternalServerError().body(format!("Erro ao obter os dados de consumo de energia: {}", error))
            }
        }
    };
}

#[post("/get-units-list")]
async fn units_list(req_body: web::Json<GetUnitListRequestBody>, globs: web::Data<Arc<GlobalVars>>) -> impl Responder {
    
    let params = GetUnitListRequestBody {
        startDate: req_body.startDate.to_string(),
        endDate: req_body.endDate.to_string(),
        units: req_body.units.clone()
    };
    
    match get_energy_units(&params, &globs){
        Ok(response) => return HttpResponse::Ok().json(response),
        Err(error) => {
            let msg_error = format!("Erro ao obter os dados das unidades na rota /get-units-list: {}", error);
            write_to_log_file_thread(&msg_error, 0, "ERROR");
            println!("{}", msg_error);
            return HttpResponse::InternalServerError().body(msg_error)
        }
    };
}

#[post("/get-energy-consumption")]
async fn get_energy_consumption(req_body: web::Json<ReqParamsGetEnergyConsumption>, globs: web::Data<Arc<GlobalVars>>) -> impl Responder {
    let start_date_formatted = format!("{} 00:00:00", req_body.start_date);
    let end_date_formatted = format!("{} 23:59:59", req_body.end_date);
    let get_hour_consumption = if req_body.get_hour_consumption.unwrap_or(false) || req_body.start_date == req_body.end_date { true } else { false };

    let mut response_day_consumption = match get_energy_day_consumption(req_body.unit_id, &start_date_formatted, &end_date_formatted, &globs) {
        Ok(res) => res,
        Err(err) => {
            let msg_error = format!("Erro ao obter os dados de consumo de energia na rota /get-energy-consumption 1: {}", err);
            write_to_log_file_thread(&msg_error, 0, "ERROR");
            println!("{}", msg_error);
            return HttpResponse::InternalServerError().body(format!("Erro ao obter consumo de energia da unidade, {} ", err))
        }
    };

    let start_date = NaiveDate::from_str(&req_body.start_date).unwrap_or_default();
    let end_date = NaiveDate::from_str(&req_body.end_date).unwrap_or_default();

    let mut day_consumption = fill_consumption_by_day(response_day_consumption, start_date, end_date, req_body.isDielUser);

    if get_hour_consumption {
        match get_energy_hours_consumption(req_body.unit_id, &start_date_formatted, &end_date_formatted, &globs) {
            Ok(res) => {
               day_consumption = adjust_consumption_by_hour(day_consumption, res);
            },
            Err(err) => {
                let msg_error = format!("Erro ao obter os dados de consumo de energia na rota /get-energy-consumption 2: {}", err);
                write_to_log_file_thread(&msg_error, 0, "ERROR");
                println!("{}", msg_error);
                return HttpResponse::InternalServerError().body(format!("Erro ao obter consumo de energia da unidade, {} ", err));
            }
        }
    } 

    HttpResponse::Ok().json(json!({ "energy_consumption_list": day_consumption }))
}

#[post("/get-procel-insights")]
async fn get_procel_insights(req_body: web::Json<GetProcelInsightsRequestBody>, globs: web::Data<Arc<GlobalVars>>) -> impl Responder {
    let params = GetProcelInsightsRequestBody {
        units: req_body.units.clone(),
        startDate: req_body.startDate.clone(),
        endDate: req_body.endDate.clone(),
        previousStartDate: req_body.previousStartDate.clone(),
        previousEndDate: req_body.previousEndDate.clone(),
        minConsumption: req_body.minConsumption.clone(),
        procelUnitsFilter: req_body.procelUnitsFilter.clone()
    };

    match procel_insigths(&params, &globs){
        Ok(response) => return HttpResponse::Ok().json(response),
        Err(error) => {
            let msg_error = format!("Erro ao obter os dados de energia na rota /get-procel-insights: {}", error);
            write_to_log_file_thread(&msg_error, 0, "ERROR");
            println!("{}", msg_error);
            return HttpResponse::InternalServerError().body(format!("Erro ao obter os dados de energia: {}", error))
        }
    };
}

#[post("/get-energy-trends")]
async fn get_energy_trends(req_body: web::Json<GetEnergyTrendsRequestBody>, globs: web::Data<Arc<GlobalVars>>) -> impl Responder {
    let params = GetEnergyTrendsRequestBody {
        units: req_body.units.clone(),
        startDate: req_body.startDate.clone(),
        endDate: req_body.endDate.clone(),
        days: req_body.days.clone(),
    };

    match energy_trends(&params, &globs){
        Ok(response) => return HttpResponse::Ok().json(response),
        Err(error) => {
            let msg_error = format!("Erro ao obter os dados de energia na rota /get-energy-trends: {}", error);
            write_to_log_file_thread(&msg_error, 0, "ERROR");
            println!("{}", msg_error);
            return HttpResponse::InternalServerError().body(format!("Erro ao obter os dados de energia: {}", error))
        }
    };
}
