use crate::filters::{Filter, FilterError};
use crate::models::Property;
use crate::ApplicationConfig;
use async_trait::async_trait;
use rusoto_core::Region;
use rusoto_dynamodb::{
  AttributeValue, BatchGetItemInput, DynamoDb, DynamoDbClient, KeysAndAttributes,
};
use std::collections::HashMap;

pub struct DynamoDbFilter {
  pub client: Option<DynamoDbClient>,
  pub existing: HashMap<String, bool>,
  pub initialized: bool,
}

impl DynamoDbFilter {
  pub fn new(_: &ApplicationConfig) -> Self {
    DynamoDbFilter {
      client: None,
      existing: HashMap::new(),
      initialized: false,
    }
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
    _: &ApplicationConfig,
    property: &Property,
    properties: &Vec<Property>,
  ) -> Result<bool, FilterError> {
    if !self.initialized {
      self.initialized = true;
      for chunks in properties.chunks(100) {
        let mut items: Vec<HashMap<String, AttributeValue>> = vec![];
        for n in chunks {
          let mut id = String::from(n.source.as_str());
          id.push('-');
          id.push_str(n.data.as_ref().unwrap().externalid.as_str());
          let mut item = HashMap::new();
          item.insert(
            String::from("id"),
            AttributeValue {
              s: Some(id),
              ..Default::default()
            },
          );
          items.push(item)
        }

        let mut tables = HashMap::new();
        tables.insert(
          String::from("properties"),
          KeysAndAttributes {
            keys: items,
            projection_expression: Some(String::from("id")),
            ..Default::default()
          },
        );

        let batch_get_input: BatchGetItemInput = BatchGetItemInput {
          request_items: tables,
          ..Default::default()
        };

        match self
          .client
          .as_ref()
          .unwrap()
          .batch_get_item(batch_get_input)
          .await
        {
          Ok(batch_get_output) => match batch_get_output.responses {
            Some(tables) => {
              println!("{:?}", tables);
              tables
                .get("properties")
                .unwrap()
                .into_iter()
                .map(|i| i.get("id").unwrap().s.as_ref().unwrap())
                .for_each(|id| {
                  self.existing.insert(id.clone(), true);
                });
            }
            None => (),
          },
          Err(e) => println!("error: {}", e.to_string()),
        }
      }
    }

    if property.data.is_some() {
      let mut id = String::from(property.source.as_str());
      id.push('-');
      id.push_str(property.data.as_ref().unwrap().externalid.as_str());

      if self.existing.contains_key(&id) {
        Ok(false)
      } else {
        self.existing.insert(id, true);
        Ok(true)
      }
    } else {
      Err(FilterError {
        message: String::from("No data!"),
      })
    }
  }
}
