use config::{Config, File};
use crate::crawlers::Config as CrawlerConfig;

#[derive(Clone, Debug)]
pub struct TelegramConfig {
  pub enabled: bool,
  pub api_key: String,
  pub chat_id: i64,
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
}

#[derive(Clone, Debug)]
pub struct ApplicationConfig {
  pub test: bool,
  pub interval: u64,
  pub thread_count: i32,
  pub geocoding: GeocodingConfig,
  pub telegram: TelegramConfig,
  pub crawler_configs: Vec<CrawlerConfig>,
  pub database: DatabaseConfig,
}

pub fn read() -> ApplicationConfig {
  let mut config = Config::new();
  config.merge(File::with_name("config")).unwrap();
  let test = config.get("test").unwrap_or(false);
  let thread_count = config.get("thread_count").unwrap_or(2);
  let interval = config.get("interval").unwrap_or(300);

  let telegram_enabled = config.get("telegram.enabled").unwrap_or(false);
  let telegram_api_key = config.get("telegram.api_key").unwrap_or(String::from(""));
  let telegram_chat_id = config.get("telegram.chat_id").unwrap_or(0);

  let geocoding_enabled = config.get("geocoding.enabled").unwrap_or(false);
  let geocoding_nominatim_url: String = config.get("geocoding.nominatim_url").unwrap_or(String::new());

  let database_enabled = config.get("database.enabled").unwrap_or(false);
  let database_auth_json_path = config.get("database.auth_json_path").unwrap_or(String::new());

  let mut crawler_configs: Vec<CrawlerConfig> = vec![];
  let watcher_arr = config.get_array("watcher").unwrap();
  for watcher in watcher_arr {
    let crawler_values = watcher.into_table().unwrap();
    let crawler = crawler_values.get("crawler").unwrap().to_owned().into_str().unwrap();
    let contract = crawler_values.get("contract_type").unwrap().to_owned().into_str().unwrap();
    let property = crawler_values.get("property_type").unwrap().to_owned().into_str().unwrap();
    let crawler_config = CrawlerConfig {
      city: crawler_values.get("city").unwrap().to_owned().into_str().unwrap(),
      address: crawler_values.get("address").unwrap().to_owned().into_str().unwrap(),
      crawler,
      contract_type: contract.parse().unwrap(),
      property_type: property.parse().unwrap(),
    };
    crawler_configs.push(crawler_config);
  }

  ApplicationConfig {
    test,
    interval,
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
    },
    crawler_configs,
  }
}
