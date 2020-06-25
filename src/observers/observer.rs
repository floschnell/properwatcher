use crate::models::Property;
use crate::ApplicationConfig;

pub struct ObserverError {
  pub message: String,
}

impl From<std::io::Error> for ObserverError {
  fn from(e: std::io::Error) -> ObserverError {
    ObserverError {
      message: format!("{}", e),
    }
  }
}

pub trait Observer {
  fn name(&self) -> String;
  fn init(&mut self, app_config: &ApplicationConfig) -> Result<(), String>;
  fn observation(
    &self,
    app_config: &ApplicationConfig,
    property: &Property,
  ) -> Result<(), ObserverError>;
}
