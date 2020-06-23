use crate::models::Property;
use crate::ApplicationConfig;

pub struct Error {
  pub message: String,
}

impl From<std::io::Error> for Error {
  fn from(e: std::io::Error) -> Error {
    Error {
      message: format!("{}", e),
    }
  }
}

pub trait Observer {
  fn name(&self) -> String;
  fn init(&mut self, app_config: &ApplicationConfig) -> Result<(), String>;
  fn observation(&self, app_config: &ApplicationConfig, property: &Property) -> Result<(), Error>;
}
