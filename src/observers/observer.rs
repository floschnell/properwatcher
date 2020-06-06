use crate::models::Property;
use crate::ApplicationConfig;

pub trait Observer {
  fn observation(&self, app_config: &ApplicationConfig, property: &Property) -> ();
}
