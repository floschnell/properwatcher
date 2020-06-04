extern crate kuchiki;
extern crate reqwest;
extern crate std;

use super::{Crawler, Error};
use crate::models::{PropertyData, PropertyType, ContractType};
use kuchiki::{ElementData, NodeDataRef};
use crate::crawlers::Metadata;
use crate::models::Encoding;

pub struct ImmoScout {}

impl Crawler for ImmoScout {
  fn metadata(&self) -> Metadata {
    Metadata {
      name: String::from("immoscout"),
      encoding: Encoding::Utf8,
    }
  }

  fn selector(&self) -> &'static str {
    "article[data-item=result]"
  }

  fn transform_result(&self, result: NodeDataRef<ElementData>) -> Result<PropertyData, Error> {
    let rent = Self::get_text(&result, ".result-list-entry__criteria dl:nth-child(1) dd")?;
    let squaremeters = Self::get_text(&result, ".result-list-entry__criteria dl:nth-child(2) dd")?;
    let rooms = Self::get_text(
      &result,
      ".result-list-entry__criteria dl:nth-child(3) dd .onlyLarge",
    )?;
    let title = Self::get_text(&result, ".result-list-entry__brand-title")?;
    let address = Self::get_text(&result, ".result-list-entry__map-link")?;
    let externalid = Self::get_attr(&result, None, "data-obid")?
      .trim()
      .to_owned();
    Ok(PropertyData {
      price: Self::parse_number(rent)?,
      squaremeters: Self::parse_number(squaremeters)?,
      address,
      title,
      rooms: Self::parse_number(rooms)?,
      externalid,
      property_type: PropertyType::Flat,
      contract_type: ContractType::Rent,
    })
  }
}
