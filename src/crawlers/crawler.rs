extern crate encoding_rs;
extern crate kuchiki;
extern crate regex;
extern crate reqwest;
extern crate std;

use self::regex::Regex;
use crate::models::Encoding;
use crate::models::PropertyData;
use kuchiki::{ElementData, NodeDataRef};
use std::ops::Deref;

#[derive(Debug)]
pub struct Error {
  pub message: String,
}

impl From<std::num::ParseFloatError> for Error {
  fn from(_err: std::num::ParseFloatError) -> Error {
    return Error {
      message: "Could not parse float!".to_owned(),
    };
  }
}

pub struct Metadata {
  pub name: String,
  pub encoding: Encoding,
}

pub trait Crawler: Send + Sync {
  fn metadata(&self) -> Metadata;

  fn selector(&self) -> &'static str;

  fn transform_result(&self, result: NodeDataRef<ElementData>) -> Result<PropertyData, Error>;

  fn get_attr(
    element: &NodeDataRef<ElementData>,
    select_opt: Option<&'static str>,
    name: &'static str,
  ) -> Result<String, Error>
  where
    Self: Sized,
  {
    match select_opt {
      Some(select) => match element.as_node().select_first(select.as_ref()) {
        Ok(node) => match node.attributes.borrow().get(name) {
          Some(val) => Ok(val.to_owned()),
          None => Err(Error {
            message: format!("Could not find attribute '{}'!", name),
          }),
        },
        Err(_e) => Err(Error {
          message: format!("Could not find an element matching selector '{}'!", select),
        }),
      },
      None => match element.deref().attributes.borrow_mut().get(name) {
        Some(val) => Ok(val.to_owned()),
        None => Err(Error {
          message: format!("Could not find attribute '{}'!", name),
        }),
      },
    }
  }

  fn get_text(result: &NodeDataRef<ElementData>, selector: &'static str) -> Result<String, Error>
  where
    Self: Sized,
  {
    match result.as_node().select_first(selector) {
      Ok(el) => Ok(el.text_contents()),
      Err(()) => Err(Error {
        message: format!("Could not find selector '{}'!", selector),
      }),
    }
  }

  fn get_texts(
    result: &NodeDataRef<ElementData>,
    selector: &'static str,
  ) -> Result<Vec<String>, Error>
  where
    Self: Sized,
  {
    match result.as_node().select(selector) {
      Ok(elements) => Ok(elements.map(|e| e.text_contents()).collect()),
      Err(()) => Err(Error {
        message: format!("Could not find selector '{}'!", selector),
      }),
    }
  }

  fn parse_number(number_as_str: String) -> Result<f32, Error>
  where
    Self: Sized,
  {
    let rent_regex = Regex::new(r"\d+(\.\d{3})*(,\d+)?").unwrap();
    match rent_regex
      .captures_iter(number_as_str.as_str())
      .next()
      .and_then(|capture| Some(capture[0].replace(".", "").replace(",", ".")))
    {
      Some(rent) => Ok(rent.parse()?),
      None => Err(Error {
        message: format!("No number found in '{}'!", number_as_str),
      }),
    }
  }

  fn log(&self, message: String) {
    println!("{}: {}", self.metadata().name, message);
  }
}
