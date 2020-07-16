use crate::models::{ContractType, Property};
use crate::observers::observer::{Observer, ObserverError};
use crate::ApplicationConfig;
use async_trait::async_trait;

pub struct DebugObserver {}

#[async_trait]
impl Observer for DebugObserver {
  fn name(&self) -> String {
    String::from("debug")
  }

  fn init(&mut self, _: &ApplicationConfig) -> Result<(), String> {
    Ok(())
  }

  async fn observation(
    &self,
    _: &ApplicationConfig,
    property: &Property,
  ) -> Result<(), ObserverError> {
    println!();
    println!(
      "Found property on {} in {}.",
      property.source, property.city
    );
    match property.data {
      Some(ref data) => {
        println!("id: {}", data.externalid);
        println!("title: {}", data.title);
        println!("address: {}", data.address);
        match data.contract_type {
          ContractType::Rent => println!("rent: {}", data.price),
          ContractType::Buy => println!("price: {}", data.price),
        }
        if data.plot_squaremeters.is_some() {
          println!(
            "squaremeters: {} ({})",
            data.squaremeters,
            data.plot_squaremeters.unwrap_or(0.0)
          );
        } else {
          println!("squaremeters: {}", data.squaremeters);
        }
        println!("rooms: {}", data.rooms);
        println!("tags: {:?}", data.tags);
        println!("url: {}", data.url);
      }
      None => (),
    }
    property.enrichments.iter().for_each(|(key, val)| {
      println!("enrichment[{}]: {}", key, val);
    });

    Ok(())
  }
}
