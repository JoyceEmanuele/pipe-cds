use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ReqParamsScriptDays {
    pub start_date: String,
    pub end_date: String,
    pub client_ids: Option<Vec<i32>>,
    pub unit_ids: Option<Vec<i32>>,
}
