use crate::models::Property;
use crate::ApplicationConfig;

pub trait Filter {
  fn name(&self) -> String;
  fn init(&mut self, app_config: &ApplicationConfig) -> Result<(), String>;
  fn filter(&self, app_config: &ApplicationConfig, property: &Property) -> bool;
}
