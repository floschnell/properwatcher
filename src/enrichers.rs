mod enricher;
mod geocoder;

pub use crate::enrichers::enricher::Enricher;
pub use crate::enrichers::geocoder::Geocoder;

pub fn get_enrichers() -> Vec<Box<dyn Enricher>> {
  vec![Box::new(Geocoder {})]
}
