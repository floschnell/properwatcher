use crate::filters::{Filter, FilterError};
use crate::models::Property;
use crate::ApplicationConfig;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct CSVProperty {
  pub source_id: String,
}

pub struct CSV {
  pub ids: Vec<String>,
}

impl CSV {
  pub fn new() -> Self {
    CSV { ids: vec![] }
  }
}

impl Filter for CSV {
  fn name(&self) -> String {
    String::from("csv")
  }

  fn init(&mut self, app_config: &ApplicationConfig) -> Result<(), String> {
    if std::path::Path::new(&app_config.csv.filename).exists() {
      let csv_reader_result = csv::Reader::from_path(&app_config.csv.filename);
      match csv_reader_result {
        Ok(mut reader) => {
          let records = reader.deserialize();
          for record in records {
            let row: CSVProperty = record.unwrap();
            self.ids.push(row.source_id);
          }
          println!(
            "loaded {} entries from csv {}",
            self.ids.len(),
            app_config.csv.filename
          );
          Ok(())
        }
        Err(e) => Err(e.to_string()),
      }
    } else {
      println!(
        "no entries loaded - csv {} does not exist yet.",
        app_config.csv.filename
      );
      Ok(())
    }
  }

  fn filter(&mut self, _: &ApplicationConfig, property: &Property) -> Result<bool, FilterError> {
    if property.data.is_some() {
      let external_id = property.data.as_ref().unwrap().externalid.clone();
      let exists = self.ids.contains(&external_id);
      if !exists {
        self.ids.push(external_id);
      }
      Ok(!exists)
    } else {
      Err(FilterError {
        message: String::from("No data!"),
      })
    }
  }
}
