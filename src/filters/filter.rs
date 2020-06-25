use crate::models::Property;
use crate::ApplicationConfig;

pub struct FilterError {
  pub message: String,
}

pub trait Filter {
  fn name(&self) -> String;
  fn init(&mut self, app_config: &ApplicationConfig) -> Result<(), String>;
  fn filter(
    &mut self,
    app_config: &ApplicationConfig,
    property: &Property,
  ) -> Result<bool, FilterError>;
}
