use crate::crawlers::Config as CrawlerConfig;
use config::{Config, File};

#[derive(Clone, Debug)]
pub struct TelegramConfig {
  pub enabled: bool,
  pub api_key: String,
  pub chat_id: String,
}

#[derive(Clone, Debug)]
pub struct GeocodingConfig {
  pub enabled: bool,
  pub nominatim_url: String,
}

#[derive(Clone, Debug)]
pub struct DatabaseConfig {
  pub enabled: bool,
  pub auth_json_path: String,
  pub collection_name: String,
}

#[derive(Clone, Debug)]
pub struct MailConfig {
  pub enabled: bool,
  pub smtp_server: String,
  pub username: String,
  pub password: String,
}

#[derive(Clone, Debug)]
pub struct ApplicationConfig {
  pub test: bool,
  pub interval: u64,
  pub initial_run: bool,
  pub thread_count: i32,
  pub geocoding: GeocodingConfig,
  pub telegram: TelegramConfig,
  pub crawler_configs: Vec<CrawlerConfig>,
  pub database: DatabaseConfig,
  pub mail: MailConfig,
}

pub fn read(config_path: String) -> ApplicationConfig {
  let mut config = Config::new();
  config.merge(File::with_name(config_path.as_str())).unwrap();
  let test = config.get("test").unwrap_or(false);
  let thread_count = config.get("thread_count").unwrap_or(2);
  let interval = config.get("interval").unwrap_or(300);
  let initial_run = config.get("initial_run").unwrap_or(false);

  let telegram_enabled = config.get("telegram.enabled").unwrap_or(false);
  let telegram_api_key = config.get("telegram.api_key").unwrap_or(String::from(""));
  let telegram_chat_id = config.get("telegram.chat_id").unwrap_or(String::from(""));

  let mail_enabled = config.get("mail.enabled").unwrap_or(false);
  let mail_smtp_server = config.get("mail.smtp_server").unwrap_or(String::from(""));
  let mail_username = config.get("mail.username").unwrap_or(String::from(""));
  let mail_password = config.get("mail.password").unwrap_or(String::from(""));

  let geocoding_enabled = config.get("geocoding.enabled").unwrap_or(false);
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
    geocoding: GeocodingConfig {
      enabled: geocoding_enabled,
      nominatim_url: geocoding_nominatim_url,
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
    crawler_configs,
  }
}
