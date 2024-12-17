use std::sync::Arc;
use rust_decimal::prelude::*;

use actix_web::{post, web, HttpResponse, Responder};
use serde_json::json;

use crate::{db::entities::energy_efficiency_hour_hist::{get_consumption_by_device_machine_unit, get_consumption_by_unit}, http::structs::energy_efficiency::ReqParamsGetTotalConsumptionByUnit, GlobalVars};

pub fn energy_efficiency_routes() -> actix_web::Scope {
    web::scope("/energy_efficiency")
    .service(get_total_consumption_by_unit)
}

#[post("/get-total-consumption-by-unit")]
async fn get_total_consumption_by_unit(req_body: web::Json<ReqParamsGetTotalConsumptionByUnit>, globs: web::Data<Arc<GlobalVars>>) -> impl Responder {
    let start_date_formatted = format!("{} 00:00:00", req_body.start_date);
    let end_date_formatted = format!("{} 23:59:59", req_body.end_date);

    let response_total_consumption = match get_consumption_by_unit(req_body.unit_id, &start_date_formatted, &end_date_formatted, &globs) {
        Ok(res) => {
            if res.is_some() {
                res.unwrap().total_refrigeration_consumption.to_f64()
            } else {
                0.0.into()
            }
        },
        Err(err) => {
            return HttpResponse::InternalServerError().body(format!("Erro ao obter consumo de refrigeração da unidade, {} ", err))
        }
    };

    let response_consumption_by_machine = match get_consumption_by_device_machine_unit(req_body.unit_id, &start_date_formatted, &end_date_formatted, &globs) {
        Ok(res) => res,
        Err(err) => {
            return HttpResponse::InternalServerError().body(format!("Erro ao obter consumo de refrigeração por máquina, {}", err))
        }
    };

    HttpResponse::Ok().json(json!({
        "total_refrigeration_consumption": response_total_consumption,
        "consumption_by_device_machine": response_consumption_by_machine,
    }))
}
