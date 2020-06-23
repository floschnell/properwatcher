mod dynamodb;
mod filter;

pub use crate::filters::dynamodb::DynamoDbFilter;
pub use crate::filters::filter::Filter;

use crate::ApplicationConfig;

pub fn get_filters(app_config: &ApplicationConfig) -> Vec<Box<dyn Filter>> {
  let filters: Vec<Box<dyn Filter>> = vec![Box::new(DynamoDbFilter::new(app_config))];
  filters
    .into_iter()
    .filter(|filter| app_config.filters.contains(&filter.name()))
    .map(|mut filter| {
      match filter.init(app_config) {
        Ok(_) => Some(filter),
        Err(_) => None,
      }
    })
    .filter(|opt_filter| opt_filter.is_some())
    .map(|opt_filter| opt_filter.unwrap())
    .collect()
}
