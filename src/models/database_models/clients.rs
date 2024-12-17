use crate::schema::clients;
use diesel::{Identifiable, Insertable, Queryable};


#[derive(Debug, Queryable, Insertable, Identifiable)]
#[table_name = "clients"]
pub struct Clients {
    pub id: Option<i32>,
    pub client_name: String,
    #[diesel(sql_type = Nullable<Integer>)]
    pub amount_minutes_check_offline: Option<i32>,
}
