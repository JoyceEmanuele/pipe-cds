use serde_json::Value;
use tokio::time;
use std::{sync::Arc, time::Duration};
use crate::{app_history::laager_hist::{LaagerConsumption, LaagerConsumptionHistoryPerHour}, models::external_models::{client::{ClientInfo, ClientListData}, device::ConfigDevices, unit::{ UnitInfo, UnitListData }}, GlobalVars};

pub struct ApiServer;

impl ApiServer {
    async fn send_request(route: &str, body_params: Option<Value>, globs: &Arc<GlobalVars>) -> Result<String, String> {
        let mut attempts = 0;
        let full_url = format!("{}{}", &globs.configfile.APISERVER_URL, route);

        loop {
    
            let body_string = match &body_params {
                Some(value) => value.to_string(),
                None => String::new(),
            };
    
            let response = reqwest::Client::new()
                .get(&full_url)
                .header("Content-Type", "application/json")
                .header("Authorization", &globs.configfile.APISERVER_TOKEN)
                .body(body_string)
                .send()
                .await;
    
            match response {
                Ok(response_body) => {
                    if response_body.status().is_success() {
                        let text = response_body.text().await.map_err(|err| format!("Erro ao ler resposta: {}", err))?;
                        return Ok(text);
                    } else {
                        attempts += 1;
                        if attempts >= 12 {
                            return Err(format!("Erro na requisição: {}, Status: {}", route, &response_body.status()));
                        } else {
                            eprintln!("Erro ao enviar requisição: {}, Tentativa {}/12, esperando 30 segundos para tentar novamente...", response_body.status(), attempts);
                            time::sleep(Duration::from_secs(30)).await;
                        }
                    
                    }
                }
                Err(err) => {
                    attempts += 1;
                    if attempts >= 12 {
                        return Err(format!("Erro ao enviar requisição após {} tentativas: {}", attempts, err));
                    } else {
                        eprintln!("Erro ao enviar requisição: {}, Tentativa {}/12, esperando 30 segundos para tentar novamente...", err, attempts);
                        time::sleep(Duration::from_secs(30)).await;
                    }
                }
            }
        }
    }

    pub async fn get_clients(client_ids: Option<Vec<i32>>, globs: &Arc<GlobalVars>) -> Result<Vec<ClientInfo>, String> {
        let route = "/clients/get-all-clients";

        let body_params = Some(serde_json::json!({
            "FILTER_BY_CLIENT_IDS": client_ids.unwrap_or([].to_vec())
        }));

        let result =  Self::send_request(route, body_params, globs).await?;

        let result_data: ClientListData = serde_json::from_str(&result).map_err(|err| format!("Erro ao desserealizar JSON, {}", err))?;

        Ok(result_data.list)
    }

    pub async fn get_all_units_by_client(client_id: &i32, units_with_others_timezones: Option<bool>, unit_ids: Option<Vec<i32>>, day: &str, globs: &Arc<GlobalVars>) -> Result<Vec<UnitInfo>, String> {
        let route = "/clients/get-all-units-by-client";
    
        let body_params = Some(serde_json::json!({
            "CLIENT_ID": client_id,
            "UNITS_WITH_OTHERS_TIMEZONES": units_with_others_timezones,
            "FILTER_BY_UNIT_IDS": unit_ids.unwrap_or([].to_vec()),
            "FILTER_BY_PRODUCTION_TIMESTAMP_DATE": day,
        }));
        
        let result = Self::send_request(route, body_params, globs).await?;

        let result_data: UnitListData = serde_json::from_str(&result).map_err(|err| format!("Erro ao desserealizar JSON, {}", err))?;

        Ok(result_data.list)
    }

    pub async fn get_config_devices(unit_id: &i32, day: &str, globs: &Arc<GlobalVars>) -> Result<ConfigDevices, String> {
        let route = "/devices/get-config-devices";
        let body_params = Some(serde_json::json!({
            "UNIT_ID": unit_id,
            "DAY": day,
        }));

        let result = Self::send_request(route, body_params, globs).await?;

        let result_data: ConfigDevices = serde_json::from_str(&result).map_err(|err| format!("Erro ao desserealizar JSON, {}", err))?;

        Ok(result_data)
    }

    pub async fn get_laager_history_list(laager_id: &str,  day: &str, globs: &Arc<GlobalVars>) -> Result<Vec<LaagerConsumptionHistoryPerHour>, String> {
        let route = "/laager/get-history-list";
    
        let body_params = Some(serde_json::json!({
            "LAAGER_CODE": laager_id,
            "FILTER_BY_HISTORY_DATE": day,
        }));

        let result = Self::send_request(&route, body_params, globs).await?;

        let result_data: LaagerConsumption = serde_json::from_str(&result).map_err(|err| format!("Erro ao desserealizar JSON, {}", err))?;
        
        Ok(result_data.history)
    }
}
