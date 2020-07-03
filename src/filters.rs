mod criteria;
mod csv;
mod dynamodb;
mod filter;

pub use crate::filters::criteria::CriteriaFilter;
pub use crate::filters::csv::CSV;
pub use crate::filters::dynamodb::DynamoDbFilter;
pub use crate::filters::filter::{Filter, FilterError};

use crate::ApplicationConfig;

pub fn get_filters(app_config: &ApplicationConfig) -> Vec<Box<dyn Filter>> {
  let filters: Vec<Box<dyn Filter>> = vec![
    Box::new(DynamoDbFilter::new(app_config)),
    Box::new(CSV::new()),
    Box::new(CriteriaFilter {}),
  ];
  filters
    .into_iter()
    .filter(|filter| app_config.filters.contains(&filter.name()))
    .map(|mut filter| match filter.init(app_config) {
      Ok(_) => Some(filter),
      Err(e) => {
        eprint!("Error while initializing filter {}: {}", &filter.name(), e);
        None
      }
    })
    .filter(|opt_filter| opt_filter.is_some())
    .map(|opt_filter| opt_filter.unwrap())
    .collect()
}
