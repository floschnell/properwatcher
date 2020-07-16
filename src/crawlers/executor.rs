extern crate encoding_rs;
extern crate kuchiki;
extern crate regex;
extern crate reqwest;
extern crate std;

use crate::crawlers::{Config, Crawler, Error as CrawlingError};
use crate::models::{Encoding, Property};
use kuchiki::iter::*;
use kuchiki::traits::*;
use reqwest::Response;
use std::time::Instant;

#[derive(Debug)]
pub struct Error {
  pub message: String,
}

impl From<CrawlingError> for Error {
  fn from(err: CrawlingError) -> Error {
    return Error {
      message: format!("Crawler Error: {}", err.message),
    };
  }
}

impl From<std::io::Error> for Error {
  fn from(_err: std::io::Error) -> Error {
    return Error {
      message: "IO Error".to_owned(),
    };
  }
}

impl From<reqwest::Error> for Error {
  fn from(_err: reqwest::Error) -> Error {
    let mut message = String::from("Request Error: ");
    message.push_str(_err.to_string().as_str());
    return Error { message };
  }
}

pub async fn execute(config: &Config, crawler: &Box<dyn Crawler>) -> Result<Vec<Property>, Error> {
  let results = get_results(config, crawler).await?;
  let mut successful: Vec<Property> = Vec::new();
  let flat_results: Vec<Result<Property, Error>> = results
    .map(|result| {
      let flat = Property::new(crawler.metadata().name.to_owned(), config.city.clone());
      let mut data = crawler.transform_result(result)?;
      data.contract_type = config.contract_type.to_owned();
      data.property_type = config.property_type.to_owned();
      Ok(flat.fill(&data))
    })
    .collect();
  for flat_result in flat_results {
    match flat_result {
      Ok(flat) => successful.push(flat),
      Err(e) => println!(
        "Could not process flat within crawler '{}', because: {}",
        crawler.metadata().name,
        e.message
      ),
    }
  }
  Ok(successful)
}

async fn decode_response(response: Response, encoding: &Encoding) -> Result<String, Error> {
  let buf: Vec<u8> = response.bytes().await.unwrap().to_vec();
  let (encoded_string, _, _) = match encoding {
    Encoding::Latin1 => encoding_rs::ISO_8859_2.decode(&buf),
    Encoding::Utf8 => encoding_rs::UTF_8.decode(&buf),
  };
  Ok(encoded_string.into())
}

async fn get_results(
  config: &Config,
  crawler: &Box<dyn Crawler>,
) -> Result<Select<Elements<Descendants>>, Error> {
  let url = config.address.to_owned();

  let request_start = Instant::now();
  crawler.log(format!(">> sending request to url '{}' ... ", url));
  let response = reqwest::get(url.as_str()).await?;
  crawler.log(format!(
    "<< received response in {} ms.",
    request_start.elapsed().as_millis()
  ));

  let parsing_start = Instant::now();
  crawler.log(format!("parsing document ..."));
  let decoded_response = decode_response(response, &crawler.metadata().encoding).await?;
  let document = kuchiki::parse_html()
    .from_utf8()
    .read_from(&mut decoded_response.as_bytes())?;
  crawler.log(format!(
    "document parsed successfully in {} ms.",
    parsing_start.elapsed().as_millis()
  ));

  match document.select(crawler.selector()) {
    Ok(nodes) => Ok(nodes),
    Err(()) => Err(Error {
      message: "Main selector did not match.".to_owned(),
    }),
  }
}
