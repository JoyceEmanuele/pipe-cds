use json5;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct ConfigFile {
  /* Credenciais para o rusthist buscar no DynamoDB as telemetrias */
  pub AWS_ACCESS_KEY_ID: String,
  pub AWS_SECRET_ACCESS_KEY: String,

  pub POSTGRES_DATABASE_URL: String,

  pub APISERVER_URL: String,
  pub APISERVER_TOKEN: String,


  pub APILAAGER_URL: String,
  pub APILAAGER_GRANT_TYPE: String,
  pub APILAAGER_CLIENT_ID: String,
  pub APILAAGER_CLIENT_SECRET: String,
  pub APILAAGER_USERNAME: String,
  pub APILAAGER_PASSWORD: String,

  /* Lista de tabelas no DynamoDB que *não* seguem o padrão de nome. As que seguem o padrão não precisam estar aqui. */
  pub CUSTOM_TABLE_NAMES_DMA: Vec<PrefixAndTable>,
  pub CUSTOM_TABLE_NAMES_DRI: Vec<PrefixAndTable>,
  pub CUSTOM_TABLE_NAMES_DUT: Vec<PrefixAndTable>,
  pub CUSTOM_TABLE_NAMES_DAC: Vec<PrefixAndTable>,
  pub CUSTOM_TABLE_NAMES_DMT: Vec<PrefixAndTable>,
  pub CUSTOM_TABLE_NAMES_DAL: Vec<PrefixAndTable>,
  pub CUSTOM_TABLE_NAMES_DAM: Vec<PrefixAndTable>,

  pub API_PORT: u16,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PrefixAndTable {
  pub dev_prefix: String,
  pub table_name: String,
}

pub fn default_configfile_path() -> String {
  "./configfile.json5".to_owned()
}

pub fn load_default_configfile() -> Result<ConfigFile, String> {
  let default_path = default_configfile_path();
  let default_missing = !std::path::Path::new(&default_path).exists();
  if default_missing {
    let example_config = include_str!("../configfile_example.json5");
    let res = json5::from_str(&example_config);
    if res.is_ok() {
      println!("Nenhum arquivo de configuração encontrado, usando configuração de exemplo");
      return res.map_err(|err| format!("{}", err));
    }
  }
  load_configfile(default_path)
}

pub fn load_configfile(path: String) -> Result<ConfigFile, String> {
  let file_contents = std::fs::read_to_string(&path).map_err(|err| format!("[{}]: {}", path, err))?;
  json5::from_str(&file_contents).map_err(|err| format!("[{}]: {}", path, err))
}
