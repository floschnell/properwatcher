extern crate kuchiki;
extern crate reqwest;
extern crate std;

use super::{Crawler, Error};
use crate::crawlers::Metadata;
use crate::models::Encoding;
use crate::models::{ContractType, PropertyData, PropertyType};
use kuchiki::{ElementData, NodeDataRef};

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
    let plot_squaremeters =
      Self::get_text(&result, ".result-list-entry__criteria dl:nth-child(4) dd")
        .map_or(None, |s| Self::parse_number(s).map_or(None, |f| Some(f)));
    let title = Self::get_text(&result, ".result-list-entry__brand-title")?
      .trim_start_matches("NEU")
      .to_string();
    let address = Self::get_text(&result, ".result-list-entry__map-link")?;
    let externalid = Self::get_attr(&result, None, "data-obid")?
      .trim()
      .to_string();
    let tags = Self::get_texts(&result, ".result-list-entry__secondary-criteria li")?
      .into_iter()
      .filter(|tag| tag != "...")
      .collect();
    Ok(PropertyData {
      price: Self::parse_number(rent)?,
      squaremeters: Self::parse_number(squaremeters)?,
      address,
      title,
      plot_squaremeters,
      rooms: Self::parse_number(rooms)?,
      url: format!("http://www.immobilienscout24.de/expose/{}", &externalid),
      externalid,
      property_type: PropertyType::Flat,
      contract_type: ContractType::Rent,
      tags,
    })
  }
}
