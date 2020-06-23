use crate::models::{ContractType, Property, PropertyType};
use crate::observers::Error;
use crate::observers::Observer;
use crate::ApplicationConfig;
use rusoto_core::Region;
use rusoto_dynamodb::{DynamoDb, DynamoDbClient, PutItemInput};
use serde_dynamodb::to_hashmap;
use serde_derive::{Serialize, Deserialize};

pub struct DynamoDbObserver {
  pub client: DynamoDbClient,
}

impl DynamoDbObserver {
  pub fn new(app_config: &ApplicationConfig) -> Self {
    DynamoDbObserver {
      client: DynamoDbClient::new(app_config.dynamodb.region.parse().unwrap_or(Region::EuCentral1)),
    }
  }
}

#[derive(Serialize, Deserialize)]
struct DynamoDbEntry {
  pub id: String,
  pub source: String,
  pub title: String,
  pub can_be_rented: bool,
  pub can_be_bought: bool,
  pub is_flat: bool,
  pub is_house: bool,
  pub date: i64,
  pub city: String,
  pub price: f32,
  pub squaremeters: f32,
  pub plot_squaremeters: Option<f32>,
  pub address: String,
  pub rooms: f32,
  pub tags: Vec<String>,
  pub latitude: Option<f32>,
  pub longitude: Option<f32>,
}

impl Observer for DynamoDbObserver {
  fn name(&self) -> String {
    String::from("dynamodb")
  }

  fn init(&mut self, _: &ApplicationConfig) -> Result<(), String> {
    Ok(())
  }

  fn observation(&self, app_config: &ApplicationConfig, property: &Property) -> Result<(), Error> {
    if property.data.is_some() {
      let property_data = property.data.as_ref().unwrap().clone();

      let mut id = String::from(property.source.as_str());
      id.push('-');
      id.push_str(property.data.as_ref().unwrap().externalid.as_str());

      let entry = DynamoDbEntry {
        id,
        source: property.source.to_owned(),
        title: property_data.title,
        address: property_data.address,
        can_be_rented: property_data.contract_type == ContractType::Rent,
        can_be_bought: property_data.contract_type == ContractType::Buy,
        is_flat: property_data.property_type == PropertyType::Flat,
        is_house: property_data.property_type == PropertyType::House,
        date: property.date,
        price: property_data.price,
        city: property.city.to_owned(),
        squaremeters: property_data.squaremeters,
        plot_squaremeters: property_data.plot_squaremeters,
        rooms: property_data.rooms,
        tags: property_data.tags.clone(),
        latitude: if property.enrichments.contains_key("latitude") {
          property
            .enrichments
            .get("latitude")
            .map(|x| x.parse().unwrap_or(0.0))
        } else {
          None
        },
        longitude: if property.enrichments.contains_key("longitude") {
          property
            .enrichments
            .get("longitude")
            .map(|x| x.parse().unwrap_or(0.0))
        } else {
          None
        },
      };

      let put_item_input: PutItemInput = PutItemInput {
        table_name: app_config.dynamodb.table_name.to_owned(),
        item: to_hashmap(&entry).unwrap(),
        ..Default::default()
      };

      let put_result = self.client.put_item(put_item_input).sync();
      match put_result {
        Ok(_) => Ok(()),
        Err(error) => {
        Err(Error {
            message: format!("Error while inserting to DynamoDb: {:?}", error)
          })
        }
      }
    } else {
      Ok(())
    }
  }
}
