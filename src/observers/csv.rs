use crate::models::Property;
use crate::observers::Observer;
use crate::ApplicationConfig;
use serde_derive::{Deserialize, Serialize};
use std::io::prelude::*;

#[derive(Serialize, Deserialize)]
struct CSVProperty {
  pub source: String,
  pub source_id: String,
  pub title: String,
  pub date: i64,
  pub city: String,
  pub price: f32,
  pub squaremeters: f32,
  pub plot_squaremeters: f32,
  pub address: String,
  pub rooms: f32,
  pub tags: String,
  pub latitude: f32,
  pub longitude: f32,
}

pub struct CSV {}

impl Observer for CSV {
  fn observation(&self, app_config: &ApplicationConfig, property: &Property) -> () {
    if app_config.csv.enabled {
      if property.data.is_some() {
        let mut file = std::fs::OpenOptions::new()
          .read(true)
          .write(true)
          .open(&app_config.csv.filename)
          .or_else(|_| std::fs::File::create(&app_config.csv.filename))
          .expect("Create file or open for reading.");

        let property_data = property.data.as_ref().unwrap();
        let csv_property = CSVProperty {
          source: property.source.to_owned(),
          source_id: property_data.externalid.to_owned(),
          title: property_data.title.to_owned(),
          date: property.date,
          city: property.city.to_owned(),
          price: property_data.price,
          squaremeters: property_data.squaremeters,
          plot_squaremeters: property_data.plot_squaremeters.unwrap_or(0.0),
          address: property_data.address.to_owned(),
          rooms: property_data.rooms,
          tags: property_data.tags.clone().join(","),
          latitude: property
            .enrichments
            .get("latitude")
            .unwrap_or(&String::from("0"))
            .parse()
            .unwrap_or(0.0),
          longitude: property
            .enrichments
            .get("longitude")
            .unwrap_or(&String::from("0"))
            .parse()
            .unwrap_or(0.0),
        };

        let mut c = std::io::Cursor::new(Vec::new());
        let buf_writer = std::io::BufWriter::new(c);
        let mut writer = csv::Writer::from_writer(buf_writer);
        writer.serialize(csv_property).expect("Write CSV");
        let x = writer.into_inner().unwrap().into_inner().unwrap();
        let mut buf_reader = std::io::BufReader::new(x);
        buf_reader.seek(std::io::SeekFrom::Start(0));
        let mut header_row = String::new();
        let mut data_row = String::new();
        buf_reader.read_line(&mut header_row);
        buf_reader.read_line(&mut data_row);

        // read first line of csv file
        let mut file_reader = std::io::BufReader::new(file);
        file_reader.seek(std::io::SeekFrom::Start(0));
        let mut first_line = String::new();
        file_reader.read_line(&mut first_line);
        let file = file_reader.into_inner();

        let mut file_writer = std::io::BufWriter::new(file);
        file_writer.seek(std::io::SeekFrom::End(0));
        if first_line.len() == 0 {
          file_writer.write(header_row.as_bytes());
        } else if header_row != first_line {
          eprintln!("header in CSV is wrong!");
          std::process::exit(1);
        }
        file_writer.write(data_row.as_bytes());
      }
    }
  }
}
