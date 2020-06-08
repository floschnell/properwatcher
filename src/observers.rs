mod firebase;
mod observer;
mod telegram;
mod mail;

pub use crate::observers::firebase::Firebase;
pub use crate::observers::observer::Observer;
pub use crate::observers::telegram::Telegram;
pub use crate::observers::mail::Mail;

use crate::ApplicationConfig;

pub fn get_observers(app_config: &ApplicationConfig) -> Vec<Box<dyn Observer>> {
  vec![Box::new(Telegram {}), Box::new(Firebase::new(app_config)), Box::new(Mail {})]
}
