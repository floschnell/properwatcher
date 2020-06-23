mod csv;
mod firebase;
mod mail;
mod observer;
mod telegram;
mod dynamodb;

pub use crate::observers::csv::CSV;
pub use crate::observers::firebase::Firebase;
pub use crate::observers::mail::Mail;
pub use crate::observers::observer::Error;
pub use crate::observers::observer::Observer;
pub use crate::observers::telegram::Telegram;
pub use crate::observers::dynamodb::DynamoDbObserver;

use crate::ApplicationConfig;

pub fn get_observers(app_config: &ApplicationConfig) -> Vec<Box<dyn Observer>> {
  let observers: Vec<Box<dyn Observer>> = vec![
    Box::new(Telegram {}),
    Box::new(Firebase::new(app_config)),
    Box::new(Mail {}),
    Box::new(CSV {}),
    Box::new(DynamoDbObserver::new(app_config)),
  ];
  observers
    .into_iter()
    .filter(|observer| app_config.observers.contains(&observer.name()))
    .map(|mut observer| {
      match observer.init(app_config) {
        Ok(_) => Some(observer),
        Err(_) => None,
      }
    })
    .filter(|opt_observer| opt_observer.is_some())
    .map(|opt_observer| opt_observer.unwrap())
    .collect()
}
