extern crate kuchiki;
extern crate regex;
extern crate reqwest;
extern crate std;

use super::{Crawler, Error};
use crate::crawlers::Metadata;
use crate::models::Encoding;
use crate::models::{ContractType, PropertyData, PropertyType};
use kuchiki::{ElementData, NodeDataRef};

pub struct ImmoWelt {
  pub brackets: regex::Regex,
}

impl ImmoWelt {
  pub fn new() -> Self {
    return ImmoWelt {
      brackets: regex::Regex::new(r"\s*\([^)]*\)").unwrap(),
    };
  }
}

impl Crawler for ImmoWelt {
  fn metadata(&self) -> Metadata {
    Metadata {
      name: String::from("immowelt"),
      encoding: Encoding::Utf8,
    }
  }

  fn selector(&self) -> &'static str {
    ".js-object[data-estateid]"
  }

  fn transform_result(&self, result: NodeDataRef<ElementData>) -> Result<PropertyData, Error> {
    let rent = Self::get_text(&result, ".hardfacts_3 .hardfact:nth-child(1) strong")?;
    let squaremeters = Self::get_text(&result, ".hardfacts_3 .hardfact:nth-child(2)")?;
    let rooms = Self::get_text(&result, ".hardfacts_3 .hardfact:nth-child(3)")?;
    let title = Self::get_text(&result, ".listcontent h2")?;
    let address = Self::get_text(&result, ".listlocation")?
      .split("\n")
      .map(|part| part.trim())
      .filter(|part| part.len() > 0)
      .collect::<Vec<_>>()
      .join(", ");
    let cleaned_address = self.brackets.replace_all(&address, "").into_owned();
    let externalid = Self::get_attr(&result, None, "data-estateid")?;
    Ok(PropertyData {
      price: Self::parse_number(rent)?,
      squaremeters: Self::parse_number(squaremeters)?,
      plot_squaremeters: None,
      address: cleaned_address,
      title,
      rooms: Self::parse_number(rooms)?,
      url: format!("https://www.immowelt.de/expose/{}", &externalid),
      externalid,
      property_type: PropertyType::Flat,
      contract_type: ContractType::Rent,
      tags: vec![],
    })
  }
}
