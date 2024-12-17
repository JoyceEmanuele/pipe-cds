
use std::{str::FromStr, sync::Arc};

use actix_web::{post, web, HttpResponse, Responder};
use chrono::{Duration, NaiveDate};

use crate::{http::structs::script_days::ReqParamsScriptDays, schedules::scheduler::run_nightly_tasks, GlobalVars};

pub fn scrip_days_route() -> actix_web::Scope {
    web::scope("/script_days")
    .service(compile_days)
    .service(compile_days_energy)
    .service(compile_days_chiller)
    .service(compile_days_water)
    .service(compile_days_energy_demand)
    .service(compile_days_energy_efficiency)
    .service(compile_days_on_outside_programming)
}

#[post("/all")]
async fn compile_days(req_body: web::Json<ReqParamsScriptDays>, globs: web::Data<Arc<GlobalVars>>) -> impl Responder {
    let start_date = NaiveDate::from_str(&req_body.start_date).unwrap_or_default();
    let end_date = NaiveDate::from_str(&req_body.end_date).unwrap_or_default();

    let num_days = (end_date - start_date).num_days() + 1;

    let mut date_array = Vec::with_capacity(num_days as usize);

    for i in 0..num_days {
        let date = start_date + Duration::days(i);
        date_array.push(date.to_string());
    }
    
    tokio::spawn(async move {
        for day in date_array {
            run_nightly_tasks(&globs, &day, None, "all", req_body.client_ids.clone(), req_body.unit_ids.clone()).await;
        }
    });

    HttpResponse::Ok().json("Task received and is being processed")
}

#[post("/energy")]
async fn compile_days_energy(req_body: web::Json<ReqParamsScriptDays>, globs: web::Data<Arc<GlobalVars>>) -> impl Responder {
    let start_date = NaiveDate::from_str(&req_body.start_date).unwrap_or_default();
    let end_date = NaiveDate::from_str(&req_body.end_date).unwrap_or_default();

    let num_days = (end_date - start_date).num_days() + 1;

    let mut date_array = Vec::with_capacity(num_days as usize);

    for i in 0..num_days {
        let date = start_date + Duration::days(i);
        date_array.push(date.to_string());
    }
    
    tokio::spawn(async move {
        for day in date_array {
            run_nightly_tasks(&globs, &day, None, "energy", req_body.client_ids.clone(), req_body.unit_ids.clone()).await;
        }
    });

    HttpResponse::Ok().json("Task received and is being processed")
}

#[post("/chiller")]
async fn compile_days_chiller(req_body: web::Json<ReqParamsScriptDays>, globs: web::Data<Arc<GlobalVars>>) -> impl Responder {
    let start_date = NaiveDate::from_str(&req_body.start_date).unwrap_or_default();
    let end_date = NaiveDate::from_str(&req_body.end_date).unwrap_or_default();

    let num_days = (end_date - start_date).num_days() + 1;

    let mut date_array = Vec::with_capacity(num_days as usize);

    for i in 0..num_days {
        let date = start_date + Duration::days(i);
        date_array.push(date.to_string());
    }
    
    tokio::spawn(async move {
        for day in date_array {
            run_nightly_tasks(&globs, &day, None, "chiller", req_body.client_ids.clone(), req_body.unit_ids.clone()).await;
        }
    });

    HttpResponse::Ok().json("Task received and is being processed")
}

#[post("/water")]
async fn compile_days_water(req_body: web::Json<ReqParamsScriptDays>, globs: web::Data<Arc<GlobalVars>>) -> impl Responder {
    let start_date = NaiveDate::from_str(&req_body.start_date).unwrap_or_default();
    let end_date = NaiveDate::from_str(&req_body.end_date).unwrap_or_default();

    let num_days = (end_date - start_date).num_days() + 1;

    let mut date_array = Vec::with_capacity(num_days as usize);

    for i in 0..num_days {
        let date = start_date + Duration::days(i);
        date_array.push(date.to_string());
    }
    
    tokio::spawn(async move {
        for day in date_array {
            run_nightly_tasks(&globs, &day, None, "water", req_body.client_ids.clone(), req_body.unit_ids.clone()).await;
        }
    });

    HttpResponse::Ok().json("Task received and is being processed")
}

#[post("/energy_efficiency")]
async fn compile_days_energy_efficiency(req_body: web::Json<ReqParamsScriptDays>, globs: web::Data<Arc<GlobalVars>>) -> impl Responder {
    let start_date = NaiveDate::from_str(&req_body.start_date).unwrap_or_default();
    let end_date = NaiveDate::from_str(&req_body.end_date).unwrap_or_default();

    let num_days = (end_date - start_date).num_days() + 1;

    let mut date_array = Vec::with_capacity(num_days as usize);

    for i in 0..num_days {
        let date = start_date + Duration::days(i);
        date_array.push(date.to_string());
    }
    
    tokio::spawn(async move {
        for day in date_array {
            run_nightly_tasks(&globs, &day, None, "energy_efficiency", req_body.client_ids.clone(), req_body.unit_ids.clone()).await;
        }
    });

    HttpResponse::Ok().json("Task received and is being processed")
}

#[post("/energy_demand")]
async fn compile_days_energy_demand(req_body: web::Json<ReqParamsScriptDays>, globs: web::Data<Arc<GlobalVars>>) -> impl Responder {
    let start_date = NaiveDate::from_str(&req_body.start_date).unwrap_or_default();
    let end_date = NaiveDate::from_str(&req_body.end_date).unwrap_or_default();

    let num_days = (end_date - start_date).num_days() + 1;

    let mut date_array = Vec::with_capacity(num_days as usize);

    for i in 0..num_days {
        let date = start_date + Duration::days(i);
        date_array.push(date.to_string());
    }
    
    tokio::spawn(async move {
        for day in date_array {
            run_nightly_tasks(&globs, &day, None, "energy_demand", req_body.client_ids.clone(), req_body.unit_ids.clone()).await;
        }
    });

    HttpResponse::Ok().json("Task received and is being processed")
}

#[post("/on_outside_programming")]
async fn compile_days_on_outside_programming(req_body: web::Json<ReqParamsScriptDays>, globs: web::Data<Arc<GlobalVars>>) -> impl Responder {
    let start_date = NaiveDate::from_str(&req_body.start_date).unwrap_or_default();
    let end_date = NaiveDate::from_str(&req_body.end_date).unwrap_or_default();

    let num_days = (end_date - start_date).num_days() + 1;

    let mut date_array = Vec::with_capacity(num_days as usize);

    for i in 0..num_days {
        let date = start_date + Duration::days(i);
        date_array.push(date.to_string());
    }
    
    tokio::spawn(async move {
        for day in date_array {
            run_nightly_tasks(&globs, &day, None, "process_unit_on_outside_programming", req_body.client_ids.clone(), req_body.unit_ids.clone()).await;
        }
    });

    HttpResponse::Ok().json("Task received and is being processed")
}