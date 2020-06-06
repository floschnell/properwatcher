mod config;
mod crawler;
mod executor;
mod immoscout;
mod immowelt;
mod sueddeutsche;
mod wggesucht;
mod wohnungsboerse;

pub use crate::crawlers::config::Config;
pub use crate::crawlers::crawler::Crawler;
pub use crate::crawlers::crawler::Error;
pub use crate::crawlers::crawler::Metadata;
pub use crate::crawlers::executor::execute;
pub use crate::crawlers::immoscout::ImmoScout;
pub use crate::crawlers::immowelt::ImmoWelt;
pub use crate::crawlers::sueddeutsche::Sueddeutsche;
pub use crate::crawlers::wggesucht::WGGesucht;
pub use crate::crawlers::wohnungsboerse::Wohnungsboerse;

pub fn get_crawlers() -> Vec<Box<dyn Crawler>> {
  vec![
    Box::new(ImmoWelt::new()),
    Box::new(WGGesucht {}),
    Box::new(Sueddeutsche::new()),
    Box::new(ImmoScout {}),
    Box::new(Wohnungsboerse {}),
  ]
}

pub fn get_crawler(name: &String) -> Result<Box<dyn Crawler>, Error> {
  for crawler in get_crawlers() {
    if crawler.metadata().name == name.to_owned() {
      return Ok(crawler);
    }
  }
  Err(Error {
    message: String::from(format!("Could not find crawler with name: {}", name)),
  })
}
