use crate::filters::Filter;
use crate::models::Property;
use crate::ApplicationConfig;
use rusoto_core::Region;
use rusoto_dynamodb::{AttributeValue, DynamoDb, DynamoDbClient, QueryInput};
use std::collections::HashMap;

pub struct DynamoDbFilter {
  pub client: DynamoDbClient,
}

impl DynamoDbFilter {
  pub fn new(app_config: &ApplicationConfig) -> Self {
    DynamoDbFilter {
      client: DynamoDbClient::new(
        app_config
          .dynamodb
          .region
          .parse()
          .unwrap_or(Region::EuCentral1),
      ),
    }
  }
}

impl Filter for DynamoDbFilter {
  fn name(&self) -> String {
    String::from("dynamodb")
  }

  fn filter(&self, app_config: &ApplicationConfig, property: &Property) -> bool {
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

      let result = self.client.query(query).sync();
      match result {
        Ok(r) => r.count.unwrap_or(0) <= 0,
        Err(e) => {
          eprintln!("Error while filtering with dynamodb: {}", e);
          true
        },
      }
    } else {
      true
    }
  }
}
