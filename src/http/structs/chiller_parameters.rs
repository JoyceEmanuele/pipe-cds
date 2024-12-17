use serde::Deserialize;

#[derive(Deserialize)]
pub struct ReqParamsGetChillerParametersHist {
    pub start_date: String,
    pub end_date: String,
    pub device_code: String,
    pub hour_graphic: bool,
}
