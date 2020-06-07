use crate::models::Property;
use crate::observers::Observer;
use crate::ApplicationConfig;
use firestore_db_and_auth::{documents, Credentials, ServiceSession};

pub struct Firebase {
  session: Option<ServiceSession>,
}

impl Firebase {
  pub fn new(app_config: &ApplicationConfig) -> Self {
    return Firebase {
      session: if app_config.database.enabled {
        print!("connecting to firebase ... ");
        let cred = Credentials::from_file(app_config.database.auth_json_path.as_str())
          .expect("Read firebase credentials file");
        println!("success.");
        Some(ServiceSession::new(cred).expect("Create firebase service account session"))
      } else {
        None
      },
    };
  }
}

impl Observer for Firebase {
  fn observation(&self, app_config: &ApplicationConfig, property: &Property) -> () {
    if app_config.database.enabled {
      let id = property.data.as_ref().map(|x| x.externalid.to_owned());
      let document_id = id.map(|x| format!("{}-{}", property.source, x));
      let result = documents::write(
        self.session.as_ref().unwrap(),
        app_config.database.collection_name.as_str(),
        document_id,
        &property,
        documents::WriteOptions::default(),
      );
      match result.err() {
        Some(error) => println!("ERROR: {:?}!", error),
        None => (),
      }
    }
  }
}