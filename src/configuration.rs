use crate::crawlers::Config as CrawlerConfig;
use config::{Config, File};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TelegramConfig {
  pub api_key: String,
  pub chat_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NominatimConfig {
  pub user_agent: String,
  pub nominatim_url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FirebaseConfig {
  pub auth_json_path: String,
  pub collection_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CSVConfig {
  pub filename: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MailConfig {
  pub smtp_server: String,
  pub username: String,
  pub password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DynamoDbConfig {
  pub table_name: String,
  pub region: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApplicationConfig {
  pub watchers: Vec<CrawlerConfig>,
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
  #[serde(default = "default_dynamodb")]
  pub dynamodb: DynamoDbConfig,
  #[serde(default = "default_nominatim")]
  pub nominatim: NominatimConfig,
  #[serde(default = "default_telegram")]
  pub telegram: TelegramConfig,
  #[serde(default = "default_firebase")]
  pub firebase: FirebaseConfig,
  #[serde(default = "default_mail")]
  pub mail: MailConfig,
  #[serde(default = "default_csv")]
  pub csv: CSVConfig,
  #[serde(default = "default_observers")]
  pub observers: Vec<String>,
  #[serde(default = "default_enrichers")]
  pub enrichers: Vec<String>,
  #[serde(default = "default_filters")]
  pub filters: Vec<String>,
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
fn default_nominatim() -> NominatimConfig {
  NominatimConfig {
    nominatim_url: String::from(""),
    user_agent: String::from("properwatcher"),
  }
}
fn default_telegram() -> TelegramConfig {
  TelegramConfig {
    api_key: String::from(""),
    chat_id: String::from(""),
  }
}
fn default_firebase() -> FirebaseConfig {
  FirebaseConfig {
    auth_json_path: String::from(""),
    collection_name: String::from(""),
  }
}
fn default_dynamodb() -> DynamoDbConfig {
  DynamoDbConfig {
    table_name: String::from("properties"),
    region: String::from("eu-central-1"),
  }
}
fn default_mail() -> MailConfig {
  MailConfig {
    smtp_server: String::from(""),
    username: String::from(""),
    password: String::from(""),
  }
}
fn default_csv() -> CSVConfig {
  CSVConfig {
    filename: String::from(""),
  }
}
fn default_observers() -> Vec<String> {
  vec![]
}
fn default_enrichers() -> Vec<String> {
  vec![]
}
fn default_filters() -> Vec<String> {
  vec![]
}

pub fn read(config_path: String) -> ApplicationConfig {
  let mut config = Config::new();
  config.merge(File::with_name(config_path.as_str())).unwrap();
  let test = config.get("test").unwrap_or(false);
  let thread_count = config.get("thread_count").unwrap_or(2);
  let interval = config.get("interval").unwrap_or(300);
  let initial_run = config.get("initial_run").unwrap_or(false);
  let run_periodically = config.get("run_periodically").unwrap_or(true);

  let telegram_api_key = config.get("telegram.api_key").unwrap_or(String::from(""));
  let telegram_chat_id = config.get("telegram.chat_id").unwrap_or(String::from(""));

  let mail_smtp_server = config.get("mail.smtp_server").unwrap_or(String::from(""));
  let mail_username = config.get("mail.username").unwrap_or(String::from(""));
  let mail_password = config.get("mail.password").unwrap_or(String::from(""));

  let csv_filename = config
    .get("csv.filename")
    .unwrap_or(String::from("properwatcher.csv"));

  let nominatim_user_agent = config
    .get("nominatim.user_agent")
    .unwrap_or(String::from("propertwatcher"));
  let nominatim_nominatim_url: String = config
    .get("nominatim.nominatim_url")
    .unwrap_or(String::new());

  let firebase_auth_json_path = config
    .get("firebase.auth_json_path")
    .unwrap_or(String::new());
  let firebase_collection_name = config
    .get("firebase.collection_name")
    .unwrap_or(String::from("properties"));

  let dynamodb_table_name = config
    .get("dynamodb.table_name")
    .unwrap_or(String::from("properties"));
  let dynamodb_region = config
    .get("dynamodb.region")
    .unwrap_or(String::from("eu-central-1"));

  let filters = config
    .get("filters")
    .unwrap_or(vec![]);
  let enrichers = config
    .get("enrichers")
    .unwrap_or(vec![]);
  let observers = config
    .get("observers")
    .unwrap_or(vec![]);

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
    nominatim: NominatimConfig {
      nominatim_url: nominatim_nominatim_url,
      user_agent: nominatim_user_agent,
    },
    telegram: TelegramConfig {
      api_key: telegram_api_key,
      chat_id: telegram_chat_id,
    },
    firebase: FirebaseConfig {
      auth_json_path: firebase_auth_json_path,
      collection_name: firebase_collection_name,
    },
    mail: MailConfig {
      smtp_server: mail_smtp_server,
      username: mail_username,
      password: mail_password,
    },
    csv: CSVConfig {
      filename: csv_filename,
    },
    dynamodb: DynamoDbConfig {
      table_name: dynamodb_table_name,
      region: dynamodb_region,
    },
    observers,
    filters,
    enrichers,
    watchers: crawler_configs,
  }
}
