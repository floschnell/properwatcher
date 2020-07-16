use crate::models::Property;
use crate::ApplicationConfig;
use async_trait::async_trait;
use std::collections::HashMap;

pub struct EnricherError {
  pub message: String,
}

#[async_trait]
pub trait Enricher {
  fn name(&self) -> String;
  fn init(&mut self, app_config: &ApplicationConfig) -> Result<(), String>;
  async fn enrich(
    &self,
    app_config: &ApplicationConfig,
    property: &Property,
  ) -> Result<HashMap<String, String>, EnricherError>;
}
