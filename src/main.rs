mod configuration;
mod crawlers;
mod geocode;
mod models;

use crate::models::{Property, PropertyData, PropertyType, ContractType};
use configuration::ApplicationConfig;
use crawlers::Config;
use std::sync::Mutex;
use std::sync::{Arc, Barrier};
use std::thread;
use std::thread::JoinHandle;
use std::time::Instant;
use firestore_db_and_auth::{Credentials, ServiceSession, documents, errors};
use std::collections::HashMap;

fn get_url(source: &String, external_id: String) -> String {
  match &source[..] {
    "immoscout" => format!("http://www.immobilienscout24.de/expose/{}", external_id),
    "immowelt" => format!("https://www.immowelt.de/expose/{}", external_id),
    "sueddeutsche" => format!("https://immobilienmarkt.sueddeutsche.de/Wohnungen/mieten/Muenchen/Wohnung/{}?comeFromTL=1", external_id),
    "wggesucht" => format!("https://www.wg-gesucht.de/{}", external_id),
    "wohnungsboerse" => format!("https://www.wohnungsboerse.net/immodetail/{}", external_id),
    _ => String::from(""),
  }
}

fn send_telegram_message(app_config: &ApplicationConfig, msg: String) -> () {
  let chat_id = app_config.telegram.chat_id;
  let api_key = &app_config.telegram.api_key;

  let client = reqwest::blocking::Client::new();
  let mut map = HashMap::new();
  map.insert("chat_id", format!("{}", chat_id));
  map.insert("text", msg);
  map.insert("parse_mode", String::from("Markdown"));

  let result = client
    .post(&format!("https://api.telegram.org/bot{}/sendMessage", api_key))
    .json(&map)
    .send();

  match result {
    Ok(response) => if response.status() != 200 {
      println!("Error while sending message: {:?}", response.text().unwrap())
    },
    Err(e) => println!("{}", e),
  }
}

fn main() {
  print!("loading configuration ... ");
  let app_config = configuration::read();
  println!("success.");

  let session: Option<ServiceSession> = if app_config.database.enabled {
    print!("connecting to firebase ... ");
    let cred = Credentials::from_file(app_config.database.auth_json_path.as_str()).expect("Read firebase credentials file");
    println!("success.");
    Some(ServiceSession::new(cred).expect("Create firebase service account session"))
  } else {
    None
  };
  let opt_session = &session.as_ref();

  let thread_count = app_config.thread_count as usize;

  if app_config.test {
    println!("----- Running in TEST mode! -----");
    let flats = vec![Property {
      city: String::from("Munich"),
      source: "immoscout".to_owned(),
      location: Some(models::Location {
        latitude: 9.0,
        longitude: 10.0,
        uncertainty: 0.0,
      }),
      data: Some(PropertyData {
        address: "Some address".to_owned(),
        externalid: "4".to_owned(),
        price: 100.0,
        rooms: 2.0,
        squaremeters: 60.0,
        title: "Test Flat".to_owned(),
        contract_type: ContractType::Rent,
        property_type: PropertyType::Flat,
      }),
      date: 0,
    }];
    println!("flat: {}", serde_json::to_string(&flats[0]).unwrap());
    send_results(&app_config, opt_session, flats);
  }

  let barrier = Arc::new(Barrier::new(thread_count + 1));
  let mut last_flats = Vec::<Property>::new();
  loop {
    println!();

    let crawl_start = Instant::now();
    let guarded_configs = Arc::new(Mutex::new(app_config.crawler_configs.to_owned()));

    // process all crawlers
    let mut thread_handles: Vec<JoinHandle<Vec<Property>>> = vec![];
    for i in 0..thread_count {
      let inner_guarded_configs = guarded_configs.clone();
      let inner_barrier = barrier.clone();
      let cap_conf = app_config.clone();
      let handle = thread::spawn(move || {
        let flats = run_thread(inner_guarded_configs, i, &cap_conf);
        inner_barrier.wait();
        flats
      });
      &mut thread_handles.push(handle);
    }

    // wait for all threads to finish
    barrier.wait();

    // collect results
    let flats = thread_handles
      .into_iter()
      .map(|h| h.join().unwrap_or_default())
      .flatten()
      .collect::<Vec<_>>();

    let run_duration = crawl_start.elapsed();
    println!(
      "analyzed {} pages and found {} flats in {}.{} seconds.",
      app_config.crawler_configs.len(),
      flats.len(),
      run_duration.as_secs(),
      run_duration.subsec_millis()
    );

    println!("Before deduplication: {}", flats.len());
    
    // filter results for duplicates
    let mut filtered_flats: Vec<_> = Vec::new();
    for current_flat in flats.to_vec() {
      let has_been_sent = last_flats
        .to_vec()
        .into_iter()
        .any(|previous_flat| previous_flat == current_flat);
      if !has_been_sent {
        filtered_flats.push(current_flat);
      }
    }

    println!("After offline deduplication: {}", filtered_flats.len());

    let new_flats = if app_config.database.enabled {
      let x = opt_session.unwrap();
      let mut new_flats: Vec<Property> = vec!();
      for ref flat in filtered_flats {
        let id = flat.data.as_ref().map(|x| x.externalid.to_owned());
        let document_id = id.map(|x| format!("{}-{}", flat.source, x)).unwrap();
        let document: Result<Property, errors::FirebaseError> = documents::read(x, "flats", document_id);
        match document {
          Ok(_) => (),
          Err(_) => new_flats.push(flat.to_owned()),
        }
      }
      println!("After online deduplication: {}", new_flats.len());
      new_flats
    } else {
      filtered_flats
    };

    // geocode all new flats
    let geocoded_flats = if app_config.geocoding.enabled {
      geocode_flats(&new_flats, &app_config)
    } else {
      new_flats
    };

    // send flats
    if app_config.test {
      for flat in geocoded_flats {
        println!("flat that would be send: {:?}", flat);
        println!("run finished.");
      }
    } else {
      send_results(&app_config, opt_session, geocoded_flats);
      println!("run finished - will now wait for {} seconds ...", app_config.interval);
    }

    // remember the flats so we can compare against them
    // during the next run ...
    last_flats = flats.to_vec();

    // pause for 5 minutes
    std::thread::sleep(std::time::Duration::from_secs(app_config.interval));
  }
}

fn run_thread(
  guarded_configs: Arc<Mutex<Vec<Config>>>,
  thread_number: usize,
  app_config: &ApplicationConfig,
) -> Vec<Property> {
  let mut flats: Vec<Property> = vec![];
  loop {
    let config_opt = guarded_configs.lock().unwrap().pop();
    match config_opt {
      Some(crawl_config) => {
        flats.append(&mut process_config(&app_config, &crawl_config, thread_number));
      }
      None => break,
    }
  }
  flats
}

fn geocode_flats(results: &Vec<Property>, config: &ApplicationConfig) -> Vec<Property> {
  let mut enriched_flats = Vec::new();
  print!("geocoding flats ...");
  for flat in results {
    let geocode_result_opt = match &flat.data {
      Some(data) => match geocode::geocode(&config.geocoding.nominatim_url, &data.address) {
        Ok(coords) => Some(coords),
        Err(_) => None,
      },
      None => None,
    };
    let enriched_flat = match geocode_result_opt {
      Some(geocode_result) => flat.locate(&geocode_result.coord, geocode_result.uncertainty),
      None => flat.clone(),
    };
    enriched_flats.push(enriched_flat);
    print!(".");
  }
  println!();
  enriched_flats
}

fn process_config(
  app_config: &ApplicationConfig,
  crawl_config: &Config,
  thread_number: usize,
) -> Vec<Property> {
  let crawler = crawlers::get_crawler(&crawl_config.crawler);
  match crawler {
    Ok(crawler) => {
      println!(
        "processing '{}' on thread {} ...",
        crawler.metadata().name,
        thread_number
      );
      let flats_result = crawlers::execute(crawl_config, &crawler);
      if flats_result.is_ok() {
        let flats = flats_result.unwrap();
        if app_config.test {
          for ref flat in &flats {
            println!("parsed flat: {:?}", flat);
          }
        }
        flats
      } else {
        eprintln!("error: {:?}", flats_result.err().unwrap().message);
        vec![]
      }
    }
    Err(e) => {
      eprintln!("config could not be processed: {:?}", e.message);
      vec![]
    }
  }
}

fn send_results(app_config: &ApplicationConfig, session: &Option<&ServiceSession>, results: Vec<Property>) {
  print!("sending flats ...");
  for flat in results {
    if app_config.database.enabled {
      let id = flat.data.as_ref().map(|x| x.externalid.to_owned());
      let document_id = id.map(|x| format!("{}-{}", flat.source, x));
      let result = documents::write(session.unwrap(), "flats", document_id, &flat, documents::WriteOptions::default());
      match result.err() {
        Some(error) => println!("ERROR: {:?}!", error),
        None => print!("."),
      }
    }

    if app_config.telegram.enabled {
      match flat.data {
        Some(property_data) => {
          let url = get_url(&flat.source, property_data.externalid);
          let property_type = match property_data.property_type {
            PropertyType::Flat => "flat",
            PropertyType::House => "house",
          };
          let contract_type = match property_data.contract_type {
            ContractType::Buy => "Buying",
            ContractType::Rent => "Renting",
          };
          send_telegram_message(app_config, format!("Hey guys, found *a new {} on {}*!\n[{}]({})\n{} the {} costs *{}€*. It has *{} rooms* and *{} sqm*.", property_type, flat.source, property_data.title, url, contract_type, property_type, property_data.price, property_data.rooms, property_data.squaremeters));
        },
        None => ()
      }
    }
  }
  println!();
}
