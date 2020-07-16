use crate::filters::{Filter, FilterError};
use crate::models::Property;
use crate::ApplicationConfig;
use async_trait::async_trait;
use rusoto_core::Region;
use rusoto_dynamodb::{AttributeValue, DynamoDb, DynamoDbClient, QueryInput};
use std::collections::HashMap;

pub struct DynamoDbFilter {
  pub client: Option<DynamoDbClient>,
}

impl DynamoDbFilter {
  pub fn new(_: &ApplicationConfig) -> Self {
    DynamoDbFilter { client: None }
  }
}

#[async_trait]
impl Filter for DynamoDbFilter {
  fn name(&self) -> String {
    String::from("dynamodb")
  }

  fn init(&mut self, app_config: &ApplicationConfig) -> Result<(), String> {
    self.client = Some(DynamoDbClient::new(
      app_config
        .dynamodb
        .region
        .parse()
        .unwrap_or(Region::EuCentral1),
    ));
    Ok(())
  }

  async fn filter(
    &mut self,
    app_config: &ApplicationConfig,
    property: &Property,
  ) -> Result<bool, FilterError> {
    if property.data.is_some() {
      let mut id = String::from(property.source.as_str());
      id.push('-');
      id.push_str(property.data.as_ref().unwrap().externalid.as_str());

      let mut values = HashMap::new();
      values.insert(
        ":id".into(),
        AttributeValue {
          s: Some(id),
          ..Default::default()
        },
      );

      let query = QueryInput {
        table_name: app_config.dynamodb.table_name.to_owned(),
        key_condition_expression: Some("id = :id".into()),
        expression_attribute_values: Some(values),
        ..Default::default()
      };

      let result = self.client.as_ref().unwrap().query(query).await;
      match result {
        Ok(r) => Ok(r.count.unwrap_or(0) <= 0),
        Err(e) => Err(FilterError {
          message: e.to_string(),
        }),
      }
    } else {
      Err(FilterError {
        message: String::from("No data!"),
      })
    }
  }
}
