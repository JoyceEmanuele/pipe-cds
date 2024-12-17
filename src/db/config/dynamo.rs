use serde_dynamo::from_items;
use std::collections::HashMap;
use rusoto_dynamodb::{AttributeValue, DynamoDb, DynamoDbClient, QueryInput, QueryError};
use rusoto_core::RusotoError;
use std::env;
use serde::{Deserialize, Serialize};



pub struct QuerierDevIdTimestamp {
    table_name: String,

    key_var_name: String, // dev_id
    part_key: String,

    order_var_name: String,
}


#[derive(Deserialize, Debug)]
pub struct PrefixAndTable {
    pub dev_prefix: String,
    pub table_name: String,
}

impl QuerierDevIdTimestamp {
    pub fn new_diel_dev(table_name: String, dev_id: String) -> Self {
        Self {
            table_name,
            key_var_name: "dev_id".to_owned(),
            order_var_name: "timestamp".to_owned(),
            part_key: dev_id,
        }
    }

    pub fn new_custom(table_name: String, key_var_name: String, order_var_name: String, part_key: String) -> Self {
        Self {
            table_name,
            key_var_name,
            order_var_name,
            part_key,
        }
    }

    async fn fetch_page(query_input: QueryInput, is_next_page: bool) -> Result<rusoto_dynamodb::QueryOutput,String> {
        let client = DynamoDbClient::new(rusoto_core::Region::UsEast1);
        let mut retries = 0;
        loop {
            match client.query(query_input.clone()).await {
                Ok(result_page) => {
                    if result_page.items.is_none() {
                        return Err("Query returned no items".to_owned());
                    }
                    return Ok(result_page);
                },
                Err(err) => {
                    match &err {
                        RusotoError::Service(QueryError::ProvisionedThroughputExceeded(err_msg)) => {
                            if (retries < 2) && (is_next_page) {
                                retries += 1;
                                eprintln!("{}", err_msg);
                                tokio::time::sleep(std::time::Duration::from_millis(2600)).await;
                                continue;
                            } else {
                                return Err(format!("ProvisionedThroughputExceeded: {}", &err_msg));
                            }
                        },
                        RusotoError::Service(QueryError::ResourceNotFound(err_msg)) => {
                            // Table not found
                            return Err(format!("ResourceNotFound: {}", &err_msg));
                        },
                        _ => return Err(err.to_string()),
                    };
                },
            };
        }
    }

    fn create_query_input(table_name: &str, key_var_name: &str, part_key: &str, order_var_name: &str, page_ts_ini: &str, ts_end: &str) -> QueryInput {
        // order_var_name = timestamp
        // key_var_name = dev_id
        // part_key = dev_id, self.serial
        // println!("dynamoQuery: {} {} {}", &table_name, &page_ts_ini, &ts_end);
        return QueryInput {
            table_name: table_name.to_owned(),
            consistent_read: Some(false),
            projection_expression: None, // Some(String::from("#ts,L1,#State,#Mode"))
            key_condition_expression: Some(format!("{key} = :{key} and #ts between :ts_begin and :ts_end", key = key_var_name)),
            // key_condition_expression: Some(format!("{key} = :{key} and begins_with(#ts, :day)", key = key_var_name)),
            expression_attribute_names: {
                let mut map = HashMap::new();
                map.insert("#ts".to_owned(), order_var_name.to_owned());
                Some(map)
            },
            expression_attribute_values: {
                let mut map = HashMap::new();
                map.insert(format!(":{}", key_var_name), AttributeValue { s: Some(part_key.to_owned()), ..AttributeValue::default() });
                map.insert(":ts_begin".to_owned(),       AttributeValue { s: Some(page_ts_ini.to_owned()), ..AttributeValue::default() });
                map.insert(":ts_end".to_owned(),         AttributeValue { s: Some(ts_end.to_owned()), ..AttributeValue::default() });
                // map.insert(":day".to_owned(),            AttributeValue { s: Some(self.day.to_string()), ..AttributeValue::default() });
                // map.insert(":ts_begin".to_owned(),     AttributeValue { s: Some(self.ts_begin.to_string()), ..AttributeValue::default() });
                // map.insert(":ts_end".to_owned(),       AttributeValue { s: Some(self.ts_end.to_string()), ..AttributeValue::default() });
                Some(map)
            },
            ..QueryInput::default()
        };
    }

    pub async fn run<'a, T, F>(&self, ts_ini: &str, ts_end: &str, proc_items: &mut F) -> Result<(), String>
    where
        T: serde::Deserialize<'a>, // serde_json::Value
        F: FnMut(Vec<T>) -> Result<(), String>,
        F: Send,
    {

        let mut query_input = Self::create_query_input(&self.table_name, &self.key_var_name, &self.part_key, &self.order_var_name, ts_ini, ts_end);
    
        let mut is_next_page = false;
        loop {
            if ts_ini >= ts_end { break; }
            let result_page = Self::fetch_page(query_input.clone(), is_next_page).await?;

            let items = result_page.items.ok_or_else(|| "ERROR 120".to_owned())?;
            let items: Vec<T> = from_items(items).map_err(|err| err.to_string())?;
            proc_items(items)?;

            if let Some(key) = result_page.last_evaluated_key {
                is_next_page = true;
                query_input.exclusive_start_key = Some(key);
                continue;
            } else {
                break;
            }
        };
        return Ok(());
    }

}
