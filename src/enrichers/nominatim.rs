use crate::enrichers::Enricher;
use crate::models::Property;
use crate::ApplicationConfig;
use reqwest::header::HeaderValue;
use reqwest::header::USER_AGENT;

extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate url;

use serde_derive::{Deserialize, Serialize};
use std::f32;
use std::num::ParseFloatError;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResult {
  pub lat: String,
  pub lon: String,
  pub boundingbox: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeocodeResult {
  pub coord: Coordinate,
  pub uncertainty: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Coordinate {
  pub latitude: f32,
  pub longitude: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BoundingBox {
  pub min_lat: f32,
  pub max_lat: f32,
  pub min_lon: f32,
  pub max_lon: f32,
}

#[derive(Debug)]
pub struct Error {
  pub message: String,
}

impl From<reqwest::Error> for Error {
  fn from(_err: reqwest::Error) -> Error {
    return Error {
      message: format!("Request Error: {}", _err),
    };
  }
}

impl From<url::ParseError> for Error {
  fn from(_err: url::ParseError) -> Error {
    return Error {
      message: "Parse Error".to_owned(),
    };
  }
}

impl From<ParseFloatError> for Error {
  fn from(_err: ParseFloatError) -> Error {
    return Error {
      message: "Number could not be parsed to float".to_owned(),
    };
  }
}

pub struct Nominatim {}

impl Enricher for Nominatim {
  fn name(&self) -> String {
    String::from("nominatim")
  }

  fn init(&mut self, _: &ApplicationConfig) -> Result<(), String> {
    Ok(())
  }

  fn enrich(&self, app_config: &ApplicationConfig, property: &Property) -> Property {
    let geocode_result_opt = match &property.data {
      Some(data) => match geocode(app_config, &data.address) {
        Ok(coords) => Some(coords),
        Err(e) => {
          println!("error during geocoding: {:?}", e);
          None
        }
      },
      None => None,
    };
    match geocode_result_opt {
      Some(geocode_result) => {
        let mut property_enriched = property.clone();
        property_enriched.enrichments.insert(
          String::from("latitude"),
          geocode_result.coord.latitude.to_string(),
        );
        property_enriched.enrichments.insert(
          String::from("longitude"),
          geocode_result.coord.longitude.to_string(),
        );
        property_enriched.enrichments.insert(
          String::from("uncertainty"),
          geocode_result.uncertainty.to_string(),
        );
        property_enriched
      }
      None => property.clone(),
    }
  }
}

pub fn geocode(app_config: &ApplicationConfig, address: &String) -> Result<GeocodeResult, Error> {
  let client = reqwest::blocking::Client::new();

  let mut url = url::Url::parse(app_config.nominatim.nominatim_url.as_str())?;
  url
    .query_pairs_mut()
    .append_pair("q", address.replace("(Kreis)", "").as_str());
  url.query_pairs_mut().append_pair("format", "json");

  let response = client
    .get(url)
    .header(
      USER_AGENT,
      HeaderValue::from_str(app_config.nominatim.user_agent.as_str()).unwrap(),
    )
    .send()?;
  let one_second = std::time::Duration::from_secs(1);
  std::thread::sleep(one_second);

  let result: Vec<ApiResult> = response.json()?;

  if result.len() >= 1 {
    let best_match: &ApiResult = result.get(0).expect("Results have been empty!");

    let bounds = match (
      best_match
        .boundingbox
        .get(0)
        .map(|c: &String| c.parse::<f32>()),
      best_match
        .boundingbox
        .get(1)
        .map(|c: &String| c.parse::<f32>()),
      best_match
        .boundingbox
        .get(2)
        .map(|c: &String| c.parse::<f32>()),
      best_match
        .boundingbox
        .get(3)
        .map(|c: &String| c.parse::<f32>()),
    ) {
      (Some(Ok(min_lat)), Some(Ok(max_lat)), Some(Ok(min_lon)), Some(Ok(max_lon))) => {
        Some(BoundingBox {
          min_lat,
          max_lat,
          min_lon,
          max_lon,
        })
      }
      _ => None,
    };

    let coord = match (best_match.lat.parse::<f32>(), best_match.lon.parse::<f32>()) {
      (Ok(latitude), Ok(longitude)) => Some(Coordinate {
        latitude,
        longitude,
      }),
      _ => None,
    };

    match (coord, bounds) {
      (Some(c), Some(b)) => Ok(GeocodeResult {
        coord: c,
        uncertainty: get_distance_from_lat_lon_in_m(b.max_lat, b.max_lon, b.min_lat, b.min_lon),
      }),
      _ => Err(Error {
        message: "Could not geocode location!".to_owned(),
      }),
    }
  } else {
    Err(Error {
      message: "Not found!".to_owned(),
    })
  }
}

fn get_distance_from_lat_lon_in_m(lat1: f32, lon1: f32, lat2: f32, lon2: f32) -> f32 {
  let earth_radius_in_m: f32 = 6371000.785;
  let d_lat: f32 = degree_to_radian(lat2 - lat1);
  let d_lon: f32 = degree_to_radian(lon2 - lon1);
  let a = (d_lat / 2.0).sin() * (d_lat / 2.0).sin()
    + degree_to_radian(lat1).cos()
      * degree_to_radian(lat2).cos()
      * (d_lon / 2.0).sin()
      * (d_lon / 2.0).sin();
  let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
  earth_radius_in_m * c
}

fn degree_to_radian(deg: f32) -> f32 {
  deg * (f32::consts::PI / 180.0)
}
