use crate::models::Property;
use crate::ApplicationConfig;

pub trait Filter {
  fn filter(&self, app_config: &ApplicationConfig, property: &Property) -> bool;
}
