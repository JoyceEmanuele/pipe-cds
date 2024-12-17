use crate::schema::machines;
use diesel::{Insertable, Queryable};

#[derive(Debug, Queryable, Insertable)]
#[table_name = "machines"]
pub struct Machines {
    pub id: Option<i32>,
    pub machine_name: String,
    pub reference_id: i32,
    pub unit_id: i32,
    #[diesel(sql_type = Nullable<Text>)]
    pub device_code_autom: Option<String>,

}
