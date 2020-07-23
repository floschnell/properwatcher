use crate::models::Property;
use crate::ApplicationConfig;
use async_trait::async_trait;

pub struct FilterError {
  pub message: String,
}

#[async_trait]
pub trait Filter {
  fn name(&self) -> String;
  fn init(&mut self, app_config: &ApplicationConfig) -> Result<(), String>;
  async fn filter(
    &mut self,
    app_config: &ApplicationConfig,
    property: &Property,
    properties: &Vec<Property>,
  ) -> Result<bool, FilterError>;
}
