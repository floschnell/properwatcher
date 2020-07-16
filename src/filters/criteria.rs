use crate::configuration::CriteriaConfig;
use crate::filters::{Filter, FilterError};
use crate::models::Property;
use crate::ApplicationConfig;
use async_trait::async_trait;

pub struct CriteriaFilter {}

#[async_trait]
impl Filter for CriteriaFilter {
  fn name(&self) -> String {
    String::from("criteria")
  }

  fn init(&mut self, app_config: &ApplicationConfig) -> Result<(), String> {
    print!("Will filter for properties ");

    match (app_config.criteria.price_min, app_config.criteria.price_max) {
      (None, None) => print!("no matter the price"),
      (Some(min), Some(max)) => print!("that cost at least {} and at most {}", min, max),
      (None, Some(max)) => print!("that cost at most {}", max),
      (Some(min), None) => print!("that cost at least {}", min),
    }

    print!(" and ");

    match (
      app_config.criteria.squaremeters_min,
      app_config.criteria.squaremeters_max,
    ) {
      (None, None) => print!("that are of any size"),
      (Some(min), Some(max)) => print!(
        "that have at least {} sqm but are smaller or equal {} sqm",
        min, max
      ),
      (None, Some(max)) => print!("that have a maximum size of {} sqm", max),
      (Some(min), None) => print!("that have at least a size of {} sqm", min),
    }

    print!(" and ");

    match (app_config.criteria.rooms_min, app_config.criteria.rooms_max) {
      (None, None) => print!("that have any number of rooms"),
      (Some(min), Some(max)) => print!("that have at least {} and at most {} rooms", min, max),
      (None, Some(max)) => print!("that have a maximum of {} rooms", max),
      (Some(min), None) => print!("that have at least {} rooms", min),
    }

    println!(".");

    Ok(())
  }

  async fn filter(
    &mut self,
    app_config: &ApplicationConfig,
    property: &Property,
  ) -> Result<bool, FilterError> {
    Ok(self.evaluate(property, &app_config.criteria))
  }
}

impl CriteriaFilter {
  fn evaluate(&self, property: &Property, criteria: &CriteriaConfig) -> bool {
    if property.data.is_none() {
      return false;
    }

    let data = &property.data.as_ref().unwrap();

    // evaluate price
    if !criteria.price_min.map_or(true, |min| data.price >= min) {
      return false;
    }
    if !criteria.price_max.map_or(true, |max| data.price <= max) {
      return false;
    }

    // evaluate rooms
    if !criteria.rooms_min.map_or(true, |min| data.rooms >= min) {
      return false;
    }
    if !criteria.rooms_max.map_or(true, |max| data.rooms <= max) {
      return false;
    }

    // evaluate squaremeters
    if !criteria
      .squaremeters_min
      .map_or(true, |min| data.squaremeters >= min)
    {
      return false;
    }
    if !criteria
      .squaremeters_max
      .map_or(true, |max| data.squaremeters <= max)
    {
      return false;
    }

    true
  }
}

#[cfg(test)]
mod tests {
  use super::CriteriaConfig;
  use super::CriteriaFilter;
  use crate::models::Property;

  #[test]
  fn filter_pass() {
    // GIVEN
    let criteria = CriteriaConfig {
      price_min: None,
      price_max: Some(100.0),
      squaremeters_min: Some(50.0),
      squaremeters_max: None,
      rooms_min: Some(2.0),
      rooms_max: None,
    };
    let filter = CriteriaFilter {};
    let property = Property::dummy(100.0, 85.0, 2.0);

    // WHEN
    let result = filter.evaluate(&property, &criteria);

    // THEN
    assert_eq!(result, true);
  }

  #[test]
  fn filter_remove() {
    // GIVEN
    let criteria = CriteriaConfig {
      price_min: None,
      price_max: Some(100.0),
      squaremeters_min: Some(50.0),
      squaremeters_max: None,
      rooms_min: Some(2.0),
      rooms_max: None,
    };
    let filter = CriteriaFilter {};
    let property = Property::dummy(200.0, 85.0, 2.0);

    // WHEN
    let result = filter.evaluate(&property, &criteria);

    // THEN
    assert_eq!(result, false);
  }
}
