mod configuration;
mod crawlers;
mod enrichers;
mod filters;
mod models;
mod observers;

use crate::crawlers::Crawler;
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
use std::time::Instant;

fn run_lambda(e: ApplicationConfig, _: Context) -> Result<Vec<Property>, HandlerError> {
  let properties = futures::executor::block_on(run(&e, true));
  Ok(properties)
}

#[tokio::main]
async fn main() {
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
    run(&app_config, !initial_run).await;
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

async fn run(app_config: &ApplicationConfig, postprocess: bool) -> Vec<Property> {
  let observers = get_observers(&app_config);
  let enrichers = get_enrichers(&app_config);
  let mut filters = get_filters(&app_config);
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
  let mut thread_handles: Vec<_> = vec![];
  for i in 0..thread_count {
    let inner_guarded_configs = guarded_configs.clone();
    let inner_barrier = barrier.clone();
    let cap_conf = app_config.clone();
    let handle = tokio::spawn(async move {
      let properties = run_thread(inner_guarded_configs, i, &cap_conf).await;
      inner_barrier.wait();
      properties
    });
    &mut thread_handles.push(handle);
  }

  // wait for all threads to finish
  barrier.wait();

  // collect results
  let x = futures::future::join_all(thread_handles).await;
  let properties: Vec<_> = futures::future::join_all(x.into_iter().map(|r| async {
    match r {
      Ok(x) => x,
      Err(_) => vec![],
    }
  }))
  .await
  .into_iter()
  .flatten()
  .collect();

  let crawl_duration = crawl_start.elapsed();
  println!(
    "analyzed {} pages and found {} properties in {}.{:03} seconds.",
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
    let mut processed_properties = vec![];

    for mut property in properties {
      let property_ref = &property;
      if futures::future::join_all(filters.iter_mut().map(|f| async move {
        match f.filter(app_config, property_ref).await {
          Ok(r) => r,
          Err(e) => {
            eprintln!("Error during filter: {}", e.message);
            true
          }
        }
      }))
      .await
      .into_iter()
      .all(std::convert::identity)
      {
        for enricher in &enrichers {
          property = match enricher.enrich(app_config, &property) {
            Ok(p) => p,
            Err(e) => {
              eprintln!(
                "Error while running enricher {}: {}",
                &enricher.name(),
                e.message
              );
              property
            }
          };
        }

        let property_ref = &property;
        futures::future::join_all(observers.iter().map(|observer| async move {
          match observer.observation(app_config, property_ref).await {
            Ok(_) => (),
            Err(e) => eprintln!(
              "Error while running observer {}: {}",
              &observer.name(),
              e.message
            ),
          }
        }))
        .await;

        processed_properties.push(property);
      }
    }

    let processing_duration = processing_start.elapsed();
    if processed_properties.len() > 0 {
      print!(" ")
    };
    println!(
      "completed in {}.{:03} seconds.",
      processing_duration.as_secs(),
      processing_duration.subsec_millis()
    );

    let run_duration = run_started.elapsed();
    println!(
      "run took {}.{:03} seconds and successfully processed {} items.",
      run_duration.as_secs(),
      run_duration.subsec_millis(),
      processed_properties.len()
    );

    processed_properties
  }
}

async fn run_thread(
  guarded_configs: Arc<Mutex<Vec<Config>>>,
  thread_number: usize,
  app_config: &ApplicationConfig,
) -> Vec<Property> {
  let crawlers: Vec<Box<dyn Crawler>> = crawlers::get_crawlers();
  let mut futures: Vec<_> = vec![];
  loop {
    let config_opt: Option<Config> = match guarded_configs.lock() {
      Ok(mut guard) => guard.pop(),
      Err(e) => {
        eprintln!(
          "Could not acquire lock on shared configurations: {}.",
          e.to_string()
        );
        continue;
      }
    };
    match config_opt {
      Some(crawl_config) => {
        let future = process_config(&crawlers, &app_config, crawl_config.clone(), thread_number);
        futures.push(Box::pin(future));
      }
      None => break,
    }
  }
  futures::future::join_all(futures)
    .await
    .into_iter()
    .flatten()
    .collect()
}

async fn process_config(
  crawlers: &Vec<Box<dyn Crawler>>,
  app_config: &ApplicationConfig,
  crawl_config: Config,
  thread_number: usize,
) -> Vec<Property> {
  let crawler = crawlers::get_crawler(&crawl_config.crawler, crawlers);
  match crawler {
    Ok(crawler) => {
      println!(
        "processing '{}' on thread {} ...",
        crawler.metadata().name,
        thread_number
      );
      let properties_result = crawlers::execute(&crawl_config, &crawler).await;
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
