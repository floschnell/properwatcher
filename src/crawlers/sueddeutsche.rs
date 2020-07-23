extern crate kuchiki;
extern crate regex;
extern crate reqwest;
extern crate std;

use super::{Crawler, Error};
use crate::crawlers::Metadata;
use crate::models::Encoding;
use crate::models::{ContractType, PropertyData, PropertyType};
use kuchiki::{ElementData, NodeDataRef};
use std::ops::Deref;

pub struct Sueddeutsche {
  pub brackets: regex::Regex,
}

impl Sueddeutsche {
  pub fn new() -> Self {
    return Sueddeutsche {
      brackets: regex::Regex::new(r"\s*\([^)]*\)").unwrap(),
    };
  }
}

impl Crawler for Sueddeutsche {
  fn metadata(&self) -> Metadata {
    Metadata {
      name: String::from("sueddeutsche"),
      encoding: Encoding::Latin1,
    }
  }

  fn selector(&self) -> &'static str {
    "#idHitContent .hitRow"
  }

  fn transform_result<'a>(&self, result: NodeDataRef<ElementData>) -> Result<PropertyData, Error> {
    let hit_rooms_div_text = Self::get_text(&result, ".hitRoomsDiv")?;
    let hit_rooms_div_elements: Vec<&str> = hit_rooms_div_text.split(", ").collect();
    let squaremeters_opt: Option<&&str> = hit_rooms_div_elements.get(0);
    let rooms_opt: Option<&&str> = hit_rooms_div_elements.get(1);

    let hit_regions_text = Self::get_text(&result, ".hitRegionTxt.hidden")?.replace("\t", "");
    let address = hit_regions_text
      .split(",")
      .skip(1)
      .flat_map(|x| vec![x, ","])
      .collect::<String>()
      .trim_end_matches(",")
      .to_string();

    let title = Self::get_text(&result, ".hitHeadline")?
      .replace("\t", "")
      .replace("\n", "");

    let rent = Self::get_text(&result, ".hitPrice")?.replace("&nbsp;", " ");

    let externalid = Self::get_attr(&result, None, "id")?.replace("idHitRowList", "");

    match (&squaremeters_opt, &rooms_opt) {
      (&Some(squaremeters), &Some(rooms)) => Ok(PropertyData {
        price: Self::parse_number(rent)?,
        squaremeters: Self::parse_number(squaremeters.deref().to_owned())?,
        plot_squaremeters: None,
        address,
        title,
        rooms: Self::parse_number(rooms.deref().to_owned())?,
        url: format!(
          "https://immobilienmarkt.sueddeutsche.de/Wohnungen/mieten/Muenchen/Wohnung/{}?comeFromTL=1",
          &externalid
        ),
        externalid,
        property_type: PropertyType::Flat,
        contract_type: ContractType::Rent,
        tags: vec![],
      }),
      _ => Err(Error {
        message: format!(
          "Information is incomplete: {:?}, {:?}!",
          squaremeters_opt, rooms_opt
        ),
      }),
    }
  }
}
