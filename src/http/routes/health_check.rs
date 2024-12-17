
use actix_web::{get, web, HttpResponse, Responder};
use serde_json::json;

pub fn health_check_route() -> actix_web::Scope {
    web::scope("/health_check")
    .service(verify_health_service)
}

#[get("/")]
async fn verify_health_service() -> impl Responder {
    HttpResponse::Ok().json(json!({"status": true}))
}
