use crate::crawlers::Config as CrawlerConfig;
use config::{Config, File};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TelegramConfig {
  pub enabled: bool,
  pub api_key: String,
  pub chat_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GeocodingConfig {
  pub enabled: bool,
  pub user_agent: String,
  pub nominatim_url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DatabaseConfig {
  pub enabled: bool,
  pub auth_json_path: String,
  pub collection_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CSVConfig {
  pub enabled: bool,
  pub filename: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MailConfig {
  pub enabled: bool,
  pub smtp_server: String,
  pub username: String,
  pub password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DynamoDbConfig {
  pub enabled: bool,
  pub table_name: String,
  pub region: String,
  pub filter_existing: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApplicationConfig {
  #[serde(default = "default_test")]
  pub test: bool,
  #[serde(default = "default_run_periodically")]
  pub run_periodically: bool,
  #[serde(default = "default_interval")]
  pub interval: u64,
  #[serde(default = "default_initial_run")]
  pub initial_run: bool,
  #[serde(default = "default_thread_count")]
  pub thread_count: i32,
  #[serde(default = "default_geocoding")]
  pub geocoding: GeocodingConfig,
  #[serde(default = "default_telegram")]
  pub telegram: TelegramConfig,
  pub watchers: Vec<CrawlerConfig>,
  #[serde(default = "default_database")]
  pub database: DatabaseConfig,
  #[serde(default = "default_mail")]
  pub mail: MailConfig,
  #[serde(default = "default_csv")]
  pub csv: CSVConfig,
  #[serde(default = "default_dynamodb")]
  pub dynamodb: DynamoDbConfig,
}

fn default_test() -> bool {
  false
}
fn default_run_periodically() -> bool {
  false
}
fn default_interval() -> u64 {
  300
}
fn default_initial_run() -> bool {
  false
}
fn default_thread_count() -> i32 {
  1
}
fn default_geocoding() -> GeocodingConfig {
  GeocodingConfig {
    enabled: false,
    nominatim_url: String::from(""),
    user_agent: String::from("properwatcher"),
  }
}
fn default_telegram() -> TelegramConfig {
  TelegramConfig {
    enabled: false,
    api_key: String::from(""),
    chat_id: String::from(""),
  }
}
fn default_database() -> DatabaseConfig {
  DatabaseConfig {
    enabled: false,
    auth_json_path: String::from(""),
    collection_name: String::from(""),
  }
}
fn default_dynamodb() -> DynamoDbConfig {
  DynamoDbConfig {
    enabled: false,
    table_name: String::from("properties"),
    region: String::from("eu-central-1"),
    filter_existing: false,
  }
}
fn default_mail() -> MailConfig {
  MailConfig {
    enabled: false,
    smtp_server: String::from(""),
    username: String::from(""),
    password: String::from(""),
  }
}
fn default_csv() -> CSVConfig {
  CSVConfig {
    enabled: false,
    filename: String::from(""),
  }
}

pub fn read(config_path: String) -> ApplicationConfig {
  let mut config = Config::new();
  config.merge(File::with_name(config_path.as_str())).unwrap();
  let test = config.get("test").unwrap_or(false);
  let thread_count = config.get("thread_count").unwrap_or(2);
  let interval = config.get("interval").unwrap_or(300);
  let initial_run = config.get("initial_run").unwrap_or(false);
  let run_periodically = config.get("run_periodically").unwrap_or(true);

  let telegram_enabled = config.get("telegram.enabled").unwrap_or(false);
  let telegram_api_key = config.get("telegram.api_key").unwrap_or(String::from(""));
  let telegram_chat_id = config.get("telegram.chat_id").unwrap_or(String::from(""));

  let mail_enabled = config.get("mail.enabled").unwrap_or(false);
  let mail_smtp_server = config.get("mail.smtp_server").unwrap_or(String::from(""));
  let mail_username = config.get("mail.username").unwrap_or(String::from(""));
  let mail_password = config.get("mail.password").unwrap_or(String::from(""));

  let csv_enabled = config.get("csv.enabled").unwrap_or(false);
  let csv_filename = config
    .get("csv.filename")
    .unwrap_or(String::from("properwatcher.csv"));

  let geocoding_enabled = config.get("geocoding.enabled").unwrap_or(false);
  let geocoding_user_agent = config
    .get("geocoding.user_agent")
    .unwrap_or(String::from("propertwatcher"));
  let geocoding_nominatim_url: String = config
    .get("geocoding.nominatim_url")
    .unwrap_or(String::new());

  let database_enabled = config.get("database.enabled").unwrap_or(false);
  let database_auth_json_path = config
    .get("database.auth_json_path")
    .unwrap_or(String::new());
  let database_collection_name = config
    .get("database.collection_name")
    .unwrap_or(String::from("properties"));

  let dynamodb_enabled = config.get("dynamodb.enabled").unwrap_or(false);
  let dynamodb_table_name = config
    .get("dynamodb.table_name")
    .unwrap_or(String::from("properties"));
  let dynamodb_region = config
    .get("dynamodb.region")
    .unwrap_or(String::from("eu-central-1"));
  let dynamodb_filter_existing = config.get("dynamodb.filter_existing").unwrap_or(false);

  let mut crawler_configs: Vec<CrawlerConfig> = vec![];
  let watcher_arr = config.get_array("watcher").unwrap();
  for watcher in watcher_arr {
    let crawler_values = watcher.into_table().unwrap();
    let crawler = crawler_values
      .get("crawler")
      .unwrap()
      .to_owned()
      .into_str()
      .unwrap();
    let contract = crawler_values
      .get("contract_type")
      .unwrap()
      .to_owned()
      .into_str()
      .unwrap();
    let property = crawler_values
      .get("property_type")
      .unwrap()
      .to_owned()
      .into_str()
      .unwrap();
    let crawler_config = CrawlerConfig {
      city: crawler_values
        .get("city")
        .unwrap()
        .to_owned()
        .into_str()
        .unwrap(),
      address: crawler_values
        .get("address")
        .unwrap()
        .to_owned()
        .into_str()
        .unwrap(),
      crawler,
      contract_type: contract.parse().unwrap(),
      property_type: property.parse().unwrap(),
    };
    crawler_configs.push(crawler_config);
  }

  ApplicationConfig {
    test,
    interval,
    initial_run,
    thread_count: thread_count,
    run_periodically,
    geocoding: GeocodingConfig {
      enabled: geocoding_enabled,
      nominatim_url: geocoding_nominatim_url,
      user_agent: geocoding_user_agent,
    },
    telegram: TelegramConfig {
      enabled: telegram_enabled,
      api_key: telegram_api_key,
      chat_id: telegram_chat_id,
    },
    database: DatabaseConfig {
      enabled: database_enabled,
      auth_json_path: database_auth_json_path,
      collection_name: database_collection_name,
    },
    mail: MailConfig {
      enabled: mail_enabled,
      smtp_server: mail_smtp_server,
      username: mail_username,
      password: mail_password,
    },
    csv: CSVConfig {
      enabled: csv_enabled,
      filename: csv_filename,
    },
    watchers: crawler_configs,
    dynamodb: DynamoDbConfig {
      enabled: dynamodb_enabled,
      table_name: dynamodb_table_name,
      region: dynamodb_region,
      filter_existing: dynamodb_filter_existing,
    },
  }
}
