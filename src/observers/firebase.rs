use crate::models::Property;
use crate::observers::Error;
use crate::observers::Observer;
use crate::ApplicationConfig;
use firestore_db_and_auth::{documents, Credentials, ServiceSession};

pub struct Firebase {
  session: ServiceSession,
}

impl Firebase {
  pub fn new(app_config: &ApplicationConfig) -> Self {
    print!("connecting to firebase ... ");
    let cred = Credentials::from_file(app_config.firebase.auth_json_path.as_str())
      .expect("Read firebase credentials file");
    println!("success.");
    let session = ServiceSession::new(cred).expect("Create firebase service account session");
    Firebase {
      session,
    }
  }
}

impl Observer for Firebase {
  fn name(&self) -> String {
    String::from("firebase")
  }

  fn observation(&self, app_config: &ApplicationConfig, property: &Property) -> Result<(), Error> {
    let id = property.data.as_ref().map(|x| x.externalid.to_owned());
    let document_id = id.map(|x| format!("{}-{}", property.source, x));
    let result = documents::write(
      &self.session,
      app_config.firebase.collection_name.as_str(),
      document_id,
      &property,
      documents::WriteOptions::default(),
    );
    match result.err() {
      Some(error) => println!("ERROR: {:?}!", error),
      None => (),
    }
    Ok(())
  }
}
