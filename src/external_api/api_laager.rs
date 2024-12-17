use std::{sync::Arc, time::Duration};

use tokio::time;

use crate::{models::external_models::device::{ LaagerLoginRequestBody, LaagerLoginResponseData, VerifyLaagerData, WaterConsumption, WaterConsumptionHistory}, GlobalVars};
use crate::{app_history::laager_hist::{LaagerConsumption, LaagerConsumptionHistoryPerHour}};

pub struct LaagerApi;

impl LaagerApi {
    async fn send_request(route: &str, globs: &Arc<GlobalVars>) -> Result<String, String> {
        let mut attempts = 0;
        let login = Self::laager_login(&globs).await?;

        loop {
            let response = reqwest::Client::new()
                .get(&format!("{}/{}", globs.configfile.APILAAGER_URL, route))
                .header("Authorization", format!("Bearer {}", login.access_token))
                .header("Accept", "application/json")
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
                            return Err(format!("Erro na requisição: {}, Status: {}", route, response_body.status()));
                        } else {
                            eprintln!("Erro na requisição: {}, Tentativa {}/12, esperando 30 segundos para tentar novamente...", response_body.status(), attempts);
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

    fn create_login_body(globs: &Arc<GlobalVars>) -> LaagerLoginRequestBody {
        LaagerLoginRequestBody {
            grant_type: globs.configfile.APILAAGER_GRANT_TYPE.clone(),
            client_id: globs.configfile.APILAAGER_CLIENT_ID.clone(),
            client_secret: globs.configfile.APILAAGER_CLIENT_SECRET.clone(),
            username: globs.configfile.APILAAGER_USERNAME.clone(),
            password: globs.configfile.APILAAGER_PASSWORD.clone(),
        }
    }
    
    async fn laager_login(globs: &Arc<GlobalVars>) -> Result<LaagerLoginResponseData, String> {
        let mut attempts = 0;
        let body = Self::create_login_body(globs);

        loop {
            let response = reqwest::Client::new()
                .post(&format!("{}/{}", globs.configfile.APILAAGER_URL, "/oauth/token"))
                .json(&body)
                .send()
                .await;
    
            match response {
                Ok(response_body) => {
                    if response_body.status().is_success() {
                        let result = response_body.text().await.map_err(|err| format!("Erro ao ler resposta: {}", err))?;
                        let result_data: LaagerLoginResponseData = serde_json::from_str(&result).map_err(|err| format!("Erro ao desserealizar JSON, {}", err))?;
                        return Ok(result_data);
                    } else {
                        attempts += 1;
                        if attempts >= 12 {
                            return Err(format!("Erro ao fazer login na API da Laager, Status: {}", response_body.status()));
                        } else {
                            eprintln!("Erro ao fazer login na API da Laager: {}, Tentativa {}/12, esperando 30 segundos para tentar novamente...", response_body.status(), attempts);
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

    pub async fn verify_laager_meter(laager_id: &str, globs: &Arc<GlobalVars>) -> Result<String, String> {
        let route = "/api/v1/leitura/agua-fria/1";

        let result = Self::send_request(route, globs).await?;

        let result_data: Vec<VerifyLaagerData> = serde_json::from_str(&result).map_err(|err| format!("Erro ao desserealizar JSON, {}", err))?;

        if let Some(meter) = result_data.iter().find(|meter| meter.customer_id == laager_id) {
            Ok (meter.rf_device_id.clone())
        } else {
            Err(format!("Medidor Laager: {} não encontrado", laager_id))
        }
    }

    pub async fn get_water_consumption(rf_device_id: String, globs: &Arc<GlobalVars>) -> Result<Vec<WaterConsumptionHistory>, String> {
        let route = format!("/api/v1/consumption/meter_details/{}", rf_device_id);

        let result = Self::send_request(&route, globs).await?;

        let result_data: WaterConsumption = serde_json::from_str(&result).map_err(|err| format!("Erro ao desserealizar JSON, {}", err))?;
        
        Ok(result_data.history)
    }

    pub async fn get_laager_consumption(rf_device_id: String, globs: &Arc<GlobalVars>) -> Result<Vec<LaagerConsumptionHistoryPerHour>, String> {
        let route = format!("/api/v1/consumption/meter_details/{}", rf_device_id);

        let result = Self::send_request(&route, globs).await?;

        let result_data: LaagerConsumption = serde_json::from_str(&result).map_err(|err| format!("Erro ao desserealizar JSON, {}", err))?;
        
        Ok(result_data.history)
    }
}
