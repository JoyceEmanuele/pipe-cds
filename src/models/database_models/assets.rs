use crate::schema::assets;
use diesel::{Insertable, Queryable};

#[derive(Debug, Queryable, Insertable)]
#[table_name = "assets"]
pub struct Assets {
    pub id: Option<i32>,
    pub asset_name: String,
    pub device_code: String,
    pub machine_reference_id: i32,
    pub reference_id: i32,
    pub unit_id: i32,

}
