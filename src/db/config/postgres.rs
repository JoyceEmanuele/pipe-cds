use diesel::{r2d2::{ConnectionManager, Pool}};
use diesel::{PgConnection};
use std::error::Error;
pub struct PostgreSQLDatabaseManager;

impl PostgreSQLDatabaseManager {
    pub fn configure_connection_pool_pg(database_url: &str) -> Result<Pool<ConnectionManager<PgConnection>>, Box<dyn Error>> {
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Pool::builder().build(manager);
        
        match &pool {
            Ok(ok) => {}
            Err(err) => {
                eprintln!("Erro ao se conectar com o PostgreSql, {}", err);
            }
        }

        Ok(pool.unwrap())
    }

}
