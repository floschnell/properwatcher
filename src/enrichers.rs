mod enricher;
mod nominatim;

pub use crate::enrichers::enricher::Enricher;
pub use crate::enrichers::nominatim::Nominatim;
use crate::ApplicationConfig;

pub fn get_enrichers(app_config: &ApplicationConfig) -> Vec<Box<dyn Enricher>> {
  let enrichers: Vec<Box<dyn Enricher>> = vec![Box::new(Nominatim {})];
  enrichers
    .into_iter()
    .filter(|enricher| app_config.enrichers.contains(&enricher.name()))
    .collect()
}
