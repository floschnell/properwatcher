use crate::filters::{Filter, FilterError};
use crate::models::Property;
use crate::ApplicationConfig;
use async_trait::async_trait;
use rusoto_core::Region;
use rusoto_dynamodb::{
  AttributeValue, BatchGetItemInput, DynamoDb, DynamoDbClient, KeysAndAttributes,
};
use std::collections::HashMap;
use tokio::time::timeout;

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
    app_config: &ApplicationConfig,
    property: &Property,
    properties: &Vec<Property>,
  ) -> Result<bool, FilterError> {
    if !self.initialized {
      self.initialized = true;
      for chunks in properties.chunks(100) {
        let mut items: Vec<HashMap<String, AttributeValue>> = vec![];
        for chunk in chunks {
          let mut id = String::from(chunk.source.as_str());
          id.push('-');
          id.push_str(chunk.data.as_ref().unwrap().externalid.as_str());
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
          app_config.dynamodb.table_name.clone(),
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

        let mut retries = 0;
        loop {
          let batch_get_out_future = self
            .client
            .as_ref()
            .unwrap()
            .batch_get_item(batch_get_input.clone());
          match timeout(std::time::Duration::from_millis(500), batch_get_out_future).await {
            Err(_) => {
              retries = retries + 1;
              eprintln!("(connection to dynamodb timed out #{})", retries);
              if retries > 2 {
                eprint!("(giving up)");
                break;
              }
            }
            Ok(Ok(batch_get_output)) => match batch_get_output.responses {
              Some(tables) => {
                tables
                  .get(&app_config.dynamodb.table_name)
                  .unwrap()
                  .into_iter()
                  .map(|i| i.get("id").unwrap().s.as_ref().unwrap().clone())
                  .for_each(|el| {
                    self.existing.insert(el.clone(), true);
                  });
                break;
              }
              None => break,
            },
            Ok(Err(e)) => {
              eprintln!("error: {}", e.to_string());
              break;
            }
          }
        }
      }
    }

    if property.data.is_some() {
      let id = property.id();

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
