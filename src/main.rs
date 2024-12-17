mod db;
mod external_api;
mod schedules;
mod schema;
mod models;
mod app_history;
mod compression;
mod telemetry_payloads;
mod configs;
mod http;

use diesel::r2d2::{self, ConnectionManager};
use std::sync::Arc;
use actix_web::{web, App, HttpServer};
use schedules::scheduler::{run_scheduler_many_days, start_scheduler, write_to_log_file_thread};
use http::routes::{chiller_parameters::chiller_parameters_routes, energy::energy_config_routes, energy_demand::energy_demand_config_routes, energy_efficiency::energy_efficiency_routes, health_check::health_check_route, script_days::scrip_days_route, water::water_config_routes};

#[derive (Clone)]
pub struct GlobalVars {
    pub configfile: configs::ConfigFile,
    pub pool: r2d2::Pool<ConnectionManager<diesel::PgConnection>>,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> std::io::Result<()> {
    check_command_line_arguments();

    let configfile = crate::configs::load_default_configfile().expect("configfile inválido");
    std::env::set_var("AWS_ACCESS_KEY_ID", &configfile.AWS_ACCESS_KEY_ID);
    std::env::set_var("AWS_SECRET_ACCESS_KEY", &configfile.AWS_SECRET_ACCESS_KEY);

    let globs = Arc::new(GlobalVars{
        configfile: configfile.clone(),
        pool: db::config::postgres::PostgreSQLDatabaseManager::configure_connection_pool_pg(&configfile.POSTGRES_DATABASE_URL.clone()).unwrap(),
    });

    let msg_init = format!("Serviço iniciado");
    write_to_log_file_thread(&msg_init, 0, "INFO");
    println!("{}", msg_init);

    let globs_for_http_server = globs.clone();
    let globs_clone = globs.clone();

    // rodará 3:01 AM em UTC e 00:01 em GMT-3 
    let _ = tokio::spawn(async move { start_scheduler(&globs, 3).await });

    // // rodará 9:01 AM em UTC e 06:01 em GMT-3 
    let _ = tokio::spawn(async move { start_scheduler(&globs_clone, 9).await });

    let _ = HttpServer::new(move || {
        let globs_for_http_server = globs_for_http_server.clone();
        App::new()
            .app_data(web::Data::new(globs_for_http_server))
            .service(energy_config_routes())
            .service(water_config_routes())
            .service(chiller_parameters_routes())
            .service(health_check_route())
            .service(scrip_days_route())
            .service(energy_efficiency_routes())
            .service(energy_demand_config_routes())
    }).bind(("0.0.0.0", configfile.API_PORT))?.run().await;

    Ok(())
}

fn check_command_line_arguments() {
    let args: Vec<String> = std::env::args().collect();
    if (args.len() >= 2) && (args[1] == "--test-config") {
        let path: String;
        if args.len() == 3 {
            path = args[2].to_owned();
        } else {
            path = crate::configs::default_configfile_path();
        }
        let result = crate::configs::load_configfile(path.clone());
        match result {
            Ok(_) => {
                println!("Arquivo de config [{:?}] OK!", path);
                std::process::exit(0);
            },
            Err(err) => {
                println!("Erro no arquivo de config: [{:?}]: {}", path, err);
                std::process::exit(1);
            },
        }
    }
}
