use crate::schema::electric_circuits;
use diesel::{Identifiable, Insertable, Queryable};


#[derive(Debug, Queryable, Insertable, Identifiable)]
#[table_name = "electric_circuits"]
pub struct ElectricCircuit {
    pub id: Option<i32>,
    pub name: String,
    pub reference_id: i32,
    pub unit_id: i32,
}
