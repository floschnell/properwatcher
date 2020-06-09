mod configuration;
mod crawlers;
mod enrichers;
mod models;
mod observers;

use crate::enrichers::get_enrichers;
use crate::models::Property;
use crate::observers::get_observers;
use configuration::ApplicationConfig;
use crawlers::Config;
use std::env;
use std::sync::Mutex;
use std::sync::{Arc, Barrier};
use std::thread;
use std::thread::JoinHandle;
use std::time::Instant;

fn main() {
  let args: Vec<String> = env::args().collect();
  let config_path: String = args
    .get(1)
    .map(|arg| arg.to_owned())
    .unwrap_or(String::from("config.toml"));

  print!("loading configuration from {} ... ", config_path);
  let app_config = configuration::read(config_path);
  println!("success.");

  let observers = get_observers(&app_config);
  let enrichers = get_enrichers();

  if app_config.test {
    println!("----- Running in TEST mode! -----");
  }

  let thread_count = app_config.thread_count as usize;
  let barrier = Arc::new(Barrier::new(thread_count + 1));
  let mut last_properties = Vec::<Property>::new();
  let mut initial_run = app_config.initial_run;
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
        let properties = run_thread(inner_guarded_configs, i, &cap_conf);
        inner_barrier.wait();
        properties
      });
      &mut thread_handles.push(handle);
    }

    // wait for all threads to finish
    barrier.wait();

    // collect results
    let properties = thread_handles
      .into_iter()
      .map(|h| h.join().unwrap_or_default())
      .flatten()
      .collect::<Vec<_>>();

    let run_duration = crawl_start.elapsed();
    println!(
      "analyzed {} pages and found {} properties in {}.{} seconds.",
      app_config.crawler_configs.len(),
      properties.len(),
      run_duration.as_secs(),
      run_duration.subsec_millis()
    );

    // filter results for duplicates
    println!("Before deduplication: {}", properties.len());
    let mut properties_deduped: Vec<_> = Vec::new();
    for current_property in properties.to_vec() {
      let has_been_sent = last_properties
        .to_vec()
        .into_iter()
        .any(|previous_property| previous_property == current_property);
      if !has_been_sent {
        properties_deduped.push(current_property);
      }
    }
    println!("After deduplication: {}", properties_deduped.len());

    if initial_run {
      println!("initial run - will not notify observers.");
      initial_run = false
    } else {
      // geocode all new properties
      let mut properties_enriched: Vec<Property> = vec![];
      for property in properties_deduped {
        for enricher in &enrichers {
          properties_enriched.push(enricher.enrich(&app_config, &property));
        }
      }

      // notify observers
      if app_config.test {
        println!("this is a test run, will not notify observers.");
        for property in properties_enriched {
          println!("found property: {:?}", property);
        }
      } else {
        for property in properties_enriched {
          for observer in &observers {
            observer.observation(&app_config, &property);
          }
        }
      }
    }

    // remember properties so we can compare against them during the next run ...
    last_properties = properties.to_vec();

    // pause for 5 minutes
    println!("run finished.");
    println!("will now wait for {} seconds ...", app_config.interval);
    std::thread::sleep(std::time::Duration::from_secs(app_config.interval));
  }
}

fn run_thread(
  guarded_configs: Arc<Mutex<Vec<Config>>>,
  thread_number: usize,
  app_config: &ApplicationConfig,
) -> Vec<Property> {
  let mut properties: Vec<Property> = vec![];
  loop {
    let config_opt = guarded_configs.lock().unwrap().pop();
    match config_opt {
      Some(crawl_config) => {
        properties.append(&mut process_config(
          &app_config,
          &crawl_config,
          thread_number,
        ));
      }
      None => break,
    }
  }
  properties
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
      let properties_result = crawlers::execute(crawl_config, &crawler);
      if properties_result.is_ok() {
        let properties = properties_result.unwrap();
        if app_config.test {
          for propety in &properties {
            println!("parsed property: {:?}", propety);
          }
        }
        properties
      } else {
        eprintln!("error: {:?}", properties_result.err().unwrap().message);
        vec![]
      }
    }
    Err(e) => {
      eprintln!("config could not be processed: {:?}", e.message);
      vec![]
    }
  }
}
