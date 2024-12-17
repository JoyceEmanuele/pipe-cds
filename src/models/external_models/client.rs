use serde::{Deserialize};

#[derive(Debug, Deserialize, Clone)]
pub struct ClientInfo {
    #[serde(rename = "CLIENT_ID")]
    pub client_id: i32,
    #[serde(rename = "CLIENT_NAME")]
    pub client_name: String,
}

#[derive(Debug, Deserialize)]
pub struct ClientListData {
    pub list: Vec<ClientInfo>,
}
