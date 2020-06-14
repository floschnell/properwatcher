use crate::models::{ContractType, Property, PropertyData, PropertyType};
use crate::observers::Error;
use crate::observers::Observer;
use crate::ApplicationConfig;
use lettre::{smtp::authentication::Credentials, SmtpClient, Transport};
use lettre_email::EmailBuilder;

use num_format::{Locale, ToFormattedString};

pub struct Mail {}

impl Observer for Mail {
  fn observation(&self, app_config: &ApplicationConfig, property: &Property) -> Result<(), Error> {
    if app_config.mail.enabled && property.data.is_some() {
      let message = build_message(property);

      let email = EmailBuilder::new()
        .to(app_config.mail.username.to_owned())
        .from(app_config.mail.username.to_owned())
        .subject(format!(
          "Found new flat: {}",
          property.data.as_ref().unwrap().title
        ))
        .html(message)
        .build();

      let creds = Credentials::new(
        app_config.mail.username.to_owned(),
        app_config.mail.password.to_owned(),
      );

      let mut mailer = SmtpClient::new_simple(app_config.mail.smtp_server.as_str())
        .unwrap()
        .credentials(creds)
        .transport();

      let result = mailer.send(email.unwrap().into());

      if !result.is_ok() {
        println!("Could not send email: {:?}", result);
      }
    }
    Ok(())
  }
}

fn build_message(property: &Property) -> String {
  let property_data: &PropertyData = property.data.as_ref().unwrap();

  let url = get_url(&property.source, property_data.externalid.to_owned());
  let property_type = match property_data.property_type {
    PropertyType::Flat => "flat",
    PropertyType::House => "house",
  };
  let contract_type = match property_data.contract_type {
    ContractType::Buy => "Buying",
    ContractType::Rent => "Renting",
  };
  let mut msg = String::from(format!(
    "Hey guys, found <b>a new {} on {}</b>!<br /><br />",
    property_type, property.source,
  ));
  msg.push_str(&format!("{}<br />", property_data.address));
  msg.push_str(&format!(
    "{} the {} costs <b>{} €</b>.<br />",
    contract_type,
    property_type,
    (property_data.price as i32).to_formatted_string(&Locale::en)
  ));
  msg.push_str(&format!(
    "It has <b>{} rooms</b> and <b>{} sqm</b>.<br />",
    property_data.rooms,
    (property_data.squaremeters as i32).to_formatted_string(&Locale::en)
  ));
  if property_data.plot_squaremeters.is_some() {
    msg.push_str(&format!(
      "Plot of land has a size of <b>{} sqm</b>.<br />",
      (property_data.plot_squaremeters.unwrap() as i32).to_formatted_string(&Locale::en),
    ));
  }
  msg.push_str("<br />");
  msg.push_str(&format!(
    "<a href='{}' target='_blank'>Find more information here ...</a>",
    url
  ));
  msg
}

fn get_url(source: &String, external_id: String) -> String {
  match &source[..] {
    "immoscout" => format!("http://www.immobilienscout24.de/expose/{}", external_id),
    "immowelt" => format!("https://www.immowelt.de/expose/{}", external_id),
    "sueddeutsche" => format!(
      "https://immobilienmarkt.sueddeutsche.de/Wohnungen/mieten/Muenchen/Wohnung/{}?comeFromTL=1",
      external_id
    ),
    "wggesucht" => format!("https://www.wg-gesucht.de/{}", external_id),
    "wohnungsboerse" => format!("https://www.wohnungsboerse.net/immodetail/{}", external_id),
    _ => String::from(""),
  }
}
