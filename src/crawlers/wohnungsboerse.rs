extern crate kuchiki;
extern crate reqwest;
extern crate std;

use super::{Crawler, Error};
use crate::crawlers::Metadata;
use crate::models::Encoding;
use crate::models::{ContractType, PropertyData, PropertyType};
use kuchiki::{ElementData, NodeDataRef};

impl From<()> for Error {
  fn from(_: ()) -> Self {
    Error {
      message: "".to_owned(),
    }
  }
}

pub struct Wohnungsboerse {}

impl Crawler for Wohnungsboerse {
  fn metadata(&self) -> Metadata {
    Metadata {
      name: String::from("wohnungsboerse"),
      encoding: Encoding::Utf8,
    }
  }

  fn selector(&self) -> &'static str {
    ".search_result_entry[class*='estate_']"
  }

  fn transform_result(&self, result: NodeDataRef<ElementData>) -> Result<PropertyData, Error> {
    let title = Self::get_text(&result, ".search_result_entry-headline")?
      .trim()
      .to_string();
    let address = Self::get_text(&result, ".search_result_entry-subheadline")?
      .trim()
      .to_string();
    let price = Self::get_attr(
      &result,
      Some("div[itemprop^=priceSpecification] meta[itemprop^=price]"),
      "content",
    )?;
    let squaremeters = Self::get_attr(
      &result,
      Some("div[itemprop^=floorSize] meta[itemprop^=value]"),
      "content",
    )?;
    let rooms = Self::get_attr(
      &result,
      Some("div[itemprop^=numberOfRooms] meta[itemprop^=value]"),
      "content",
    )?;
    let link = Self::get_attr(&result, Some(".search_result_entry-headline a"), "href")?;
    let externalid_opt = link.rsplit("/").next();

    match externalid_opt {
      Some(externalid) => Ok(PropertyData {
        price: Self::parse_number(price)?,
        squaremeters: Self::parse_number(squaremeters)?,
        plot_squaremeters: None,
        address,
        title,
        rooms: Self::parse_number(rooms)?,
        url: format!("https://www.wohnungsboerse.net/immodetail/{}", &externalid),
        externalid: externalid.to_string(),
        property_type: PropertyType::Flat,
        contract_type: ContractType::Rent,
        tags: vec![],
      }),
      None => Err(Error {
        message: "Could not find an external id".to_string(),
      }),
    }
  }
}
