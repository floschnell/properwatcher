extern crate kuchiki;
extern crate reqwest;
extern crate std;

use super::{Crawler, Error};
use crate::crawlers::Metadata;
use crate::models::Encoding;
use crate::models::{ContractType, PropertyData, PropertyType};
use kuchiki::{ElementData, NodeDataRef};

pub struct WGGesucht {}

impl Crawler for WGGesucht {
  fn metadata(&self) -> Metadata {
    Metadata {
      name: String::from("wggesucht"),
      encoding: Encoding::Utf8,
    }
  }

  fn selector(&self) -> &'static str {
    "tr[adid^=wohnungen]"
  }

  fn transform_result(&self, result: NodeDataRef<ElementData>) -> Result<PropertyData, Error> {
    let only_limited = Self::get_text(&result, ".ang_spalte_freibis")?.trim().len() > 0;
    if only_limited {
      Err(Error {
        message: "Flat is only available for a limited time.".to_owned(),
      })
    } else {
      let rent = Self::get_text(&result, ".ang_spalte_miete")?;
      let squaremeters = Self::get_text(&result, ".ang_spalte_groesse")?;
      let rooms = Self::get_text(&result, ".ang_spalte_zimmer")?;
      let title = "Wohnung auf WG Gesucht".to_owned();
      let address = "München, ".to_owned()
        + Self::get_text(&result, ".ang_spalte_stadt")?
          .replace("\n", "")
          .trim();
      let externalid = Self::get_attr(&result, None, "adid")?;
      Ok(PropertyData {
        price: Self::parse_number(rent)?,
        squaremeters: Self::parse_number(squaremeters)?,
        plot_squaremeters: None,
        address,
        title,
        rooms: Self::parse_number(rooms)?,
        url: format!("https://www.wg-gesucht.de/{}", &externalid),
        externalid,
        property_type: PropertyType::Flat,
        contract_type: ContractType::Rent,
        tags: vec![],
      })
    }
  }
}
