use biopoem_api::{self, client};
use factotum::{execute_dag, is_valid_url};
use log::LevelFilter;
use log4rs;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use poem::{
  error::NotFoundError, http::StatusCode, listener::TcpListener, EndpointExt, Response, Server,
};
use std::error::Error;
use std::fs::{self, File};
use std::io::{copy, Write};
use std::path::Path;
use std::{env, process};
use structopt::StructOpt;

/// Client for Biopoem
#[derive(StructOpt, PartialEq, Debug)]
#[structopt(setting=structopt::clap::AppSettings::ColoredHelp, name="Biopoem - Client", author="Jingcheng Yang <yjcyxky@163.com>")]
pub struct Arguments {
  /// Which working directory for saving data.
  #[structopt(name = "workdir", short = "w", long = "workdir", default_value = ".")]
  workdir: String,

  /// 127.0.0.1 or 0.0.0.0
  #[structopt(name = "host", short = "H", long = "host", possible_values=&["127.0.0.1", "0.0.0.0"], default_value = "127.0.0.1")]
  host: String,

  /// Which port.
  #[structopt(name = "port", short = "p", long = "port", default_value = "3000")]
  port: String,

  /// Url of the dag file.
  #[structopt(name = "dag", short = "d", long = "dag")]
  dag: String,

  /// Url of the dag file.
  #[structopt(name = "webhook", short = "W", long = "webhook")]
  webhook: String,
}

pub async fn download_dag_file(dag_file_url: &str, destfile: &str) {
  // Download dag file
  let mut dest = if Path::new(destfile).exists() {
    File::options().write(true).open(destfile).unwrap()
  } else {
    File::create(destfile).unwrap()
  };
  let response = reqwest::get(dag_file_url).await.unwrap();
  let content = response.text().await.unwrap();
  copy(&mut content.as_bytes(), &mut dest).unwrap();
  info!(target:"stdout", "Save {} to {}", dag_file_url, destfile);
}

fn init_logger() -> Result<log4rs::Handle, String> {
  let stdout = ConsoleAppender::builder()
    .encoder(Box::new(PatternEncoder::new(
      "[Client] {d} - {l} -{t} - {m}{n}",
    )))
    .build();

  match fs::remove_file("client.log") {
    _ => {}
  };

  let file = FileAppender::builder()
    .encoder(Box::new(PatternEncoder::new(
      "[File] {d} - {l} - {t} - {m}{n}",
    )))
    .build("client.log")
    .unwrap();

  let config = Config::builder()
    .appender(Appender::builder().build("stdout", Box::new(stdout)))
    .appender(Appender::builder().build("file", Box::new(file)))
    .logger(
      Logger::builder()
        .appender("stdout")
        .additive(false)
        .build("stdout", LevelFilter::Info),
    )
    .build(Root::builder().appender("file").build(LevelFilter::Info))
    .unwrap();

  log4rs::init_config(config).map_err(|e| {
    format!(
      "couldn't initialize log configuration. Reason: {}",
      e.description()
    )
  })
}

#[tokio::main]
pub async fn run(args: &Arguments) {
  let workdir = &args.workdir;
  if !Path::new(&workdir).exists() {
    match fs::create_dir(workdir) {
      Err(err) => {
        println!("Cannot create the directory {}, {}", workdir, err);
        process::exit(biopoem_api::PROC_OTHER_ERROR);
      }
      Ok(_) => {
        println!("Create the directory {}", workdir);
      }
    };
  }

  match env::set_current_dir(&workdir) {
    Err(msg) => {
      println!("Cannot set working directory {}.", &msg);
      process::exit(biopoem_api::PROC_OTHER_ERROR);
    }
    _ => {}
  };

  if let Err(log) = init_logger() {
    error!(target:"stdout", "Log initialization error, {}", log);
    process::exit(biopoem_api::PROC_OTHER_ERROR);
  };

  let destfile = "dag.factfile";
  match is_valid_url(&args.dag) {
    Err(_) => {
      if !Path::new(&args.dag).exists() {
        error!(target:"stdout", "dag argument ({}) is not valid, must be a http(s):// link or a local file.", &args.dag);
        process::exit(biopoem_api::PROC_OTHER_ERROR);
      } else {
        if destfile != &args.dag {
          fs::copy(&args.dag, destfile).unwrap();
        }
      }
    }
    _ => {
      download_dag_file(&args.dag, destfile).await;
    }
  }

  let dag = args.dag.clone();
  let webhook_url = args.webhook.clone();
  tokio::spawn(async move {
    info!(target:"stdout", "Launch DAG engine with {}", &dag);
    let exit_code = execute_dag(destfile, Some(webhook_url));

    let statusfile = "status";

    let status = match exit_code == 0 {
      false => "Failed",
      true => "Success",
    };

    match fs::write(statusfile, status.to_string()) {
      Err(msg) => error!("Cannot write status, {}", msg),
      _ => {}
    };
  });

  info!(target:"stdout", "Launch client on {}:{}", &args.host[..], &args.port[..]);
  let route = client::route::init_route().catch_error(|err: NotFoundError| async {
    Response::builder()
      .status(StatusCode::NOT_FOUND)
      .body("Not found")
  });

  if let Err(err) = Server::new(TcpListener::bind(format!("{}:{}", args.host, args.port)))
    .run(route)
    .await
  {
    error!("{}", err);
    process::exit(biopoem_api::PROC_EXEC_ERROR);
  }
}
