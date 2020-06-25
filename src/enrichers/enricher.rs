use crate::models::Property;
use crate::ApplicationConfig;

pub struct EnricherError {
  pub message: String,
}

pub trait Enricher {
  fn name(&self) -> String;
  fn init(&mut self, app_config: &ApplicationConfig) -> Result<(), String>;
  fn enrich(
    &self,
    app_config: &ApplicationConfig,
    property: &Property,
  ) -> Result<Property, EnricherError>;
}
