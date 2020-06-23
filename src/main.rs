mod configuration;
mod crawlers;
mod enrichers;
mod filters;
mod models;
mod observers;

use crate::enrichers::get_enrichers;
use crate::filters::get_filters;
use crate::models::Property;
use crate::observers::get_observers;
use configuration::ApplicationConfig;
use crawlers::Config;
use lambda_runtime::{error::HandlerError, lambda, Context};
use std::env;
use std::io::prelude::*;
use std::sync::Mutex;
use std::sync::{Arc, Barrier};
use std::thread;
use std::thread::JoinHandle;
use std::time::Instant;

fn run_lambda(e: ApplicationConfig, _: Context) -> Result<Vec<Property>, HandlerError> {
  let properties = run(&e, true);
  Ok(properties)
}

fn main() {
  let args: Vec<String> = env::args().collect();

  if env::var("AWS_LAMBDA_FUNCTION_NAME").is_ok() {
    println!("Running lambda ...");
    lambda!(run_lambda);
  }

  let config_path: String = args
    .get(1)
    .map(|arg| arg.to_owned())
    .unwrap_or(String::from("config.toml"));

  print!("loading configuration from {} ... ", config_path);
  let app_config = configuration::read(config_path);
  println!("success.");

  let mut initial_run = app_config.initial_run;
  loop {
    run(&app_config, !initial_run);
    if initial_run {
      initial_run = false;
      println!("initial run finished.");
    } else {
      println!("run finished.");
    }

    // pause until next run
    if app_config.run_periodically {
      println!("will now wait for {} seconds ...", app_config.interval);
      std::thread::sleep(std::time::Duration::from_secs(app_config.interval));
    } else {
      break;
    }
  }
}

fn run(app_config: &ApplicationConfig, postprocess: bool) -> Vec<Property> {
  let observers = get_observers(&app_config);
  let enrichers = get_enrichers(&app_config);
  let filters = get_filters(&app_config);
  let run_started = Instant::now();

  let observer_names: Vec<String> = observers.iter().map(|o| o.name()).collect();
  let filter_names: Vec<String> = filters.iter().map(|f| f.name()).collect();
  let enricher_names: Vec<String> = enrichers.iter().map(|e| e.name()).collect();
  println!();
  println!("starting run.");
  println!("active filters: {:?}", filter_names);
  println!("active enrichers: {:?}", enricher_names);
  println!("active observers: {:?}", observer_names);

  if app_config.test {
    println!("----- Running in TEST mode! -----");
  }

  let thread_count = app_config.thread_count as usize;
  let barrier = Arc::new(Barrier::new(thread_count + 1));

  let crawl_start = Instant::now();
  let guarded_configs = Arc::new(Mutex::new(app_config.watchers.to_owned()));

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
  let mut properties = thread_handles
    .into_iter()
    .map(|h| h.join().unwrap_or_default())
    .flatten()
    .collect::<Vec<_>>();

  let crawl_duration = crawl_start.elapsed();
  println!(
    "analyzed {} pages and found {} properties in {}.{} seconds.",
    app_config.watchers.len(),
    properties.len(),
    crawl_duration.as_secs(),
    crawl_duration.subsec_millis()
  );

  if !postprocess {
    println!("will not process properties.");
    properties
  } else {

    print!("processing properties ");
    let _ = std::io::stdout().flush();
    let processing_start = Instant::now();
    properties = properties
      .into_iter()
      .inspect(|_| {
        print!(".");
        let _ = std::io::stdout().flush();
      })
      .filter(|property|
        filters
          .iter()
          .all(|f| f.filter(app_config, property)))
      .map(|property|
        enrichers
          .iter()
          .fold(property, |property, enricher|
            enricher.enrich(app_config, &property)))
      .inspect(|property|
        if app_config.test {
          println!("{:?}", property);
        } else {
          observers
          .iter()
          .for_each(|observer| {
            match observer.observation(app_config, property) {
              Ok(_) => (),
              Err(e) => eprintln!("Error during observer {}: {}", &observer.name(), e.message),
            }
          })
        })
      .collect();
    let processing_duration = processing_start.elapsed();
    println!(" completed in {}.{} seconds.", processing_duration.as_secs(), processing_duration.subsec_millis());

    let run_duration = run_started.elapsed();
    println!("run took {}.{} seconds.", run_duration.as_secs(), run_duration.subsec_millis());

    properties
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
