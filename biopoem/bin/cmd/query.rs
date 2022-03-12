use biopoem_api::{server, server::dag, server::remote};
use chrono;
use log::LevelFilter;
use log4rs;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use prettytable::{Cell, Row, Table};
use reqwest;
use std::error::Error;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;
use std::{env, fs, process};
use structopt::StructOpt;
use tokio::{self, time};

/// Query Task Status for Biopoem
#[derive(StructOpt, PartialEq, Debug)]
#[structopt(setting=structopt::clap::AppSettings::ColoredHelp, name="Biopoem - Query", author="Jingcheng Yang <yjcyxky@163.com>")]
pub struct Arguments {
  /// The host file.
  #[structopt(name = "hosts", short = "-H", long = "hosts", default_value = "hosts")]
  hosts: String,

  /// The monitoring mode.
  #[structopt(name = "online", short = "-o", long = "online")]
  online: bool,

  /// The monitoring interval, minutes.
  #[structopt(
    name = "interval",
    short = "-i",
    long = "interval",
    default_value = "1"
  )]
  interval: u64,
}

fn init_logger() -> Result<log4rs::Handle, String> {
  let stdout = ConsoleAppender::builder()
    .encoder(Box::new(PatternEncoder::new(
      "[Query] {d} - {l} -{t} - {m}{n}",
    )))
    .build();

  let config = Config::builder()
    .appender(Appender::builder().build("stdout", Box::new(stdout)))
    .logger(
      Logger::builder()
        .appender("stdout")
        .additive(false)
        .build("stdout", LevelFilter::Info),
    )
    .build(Root::builder().appender("stdout").build(LevelFilter::Info))
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
  if let Err(log) = init_logger() {
    error!(target:"stdout", "Log initialization error, {}", log);
    process::exit(biopoem_api::PROC_OTHER_ERROR);
  };

  let hosts = server::host::read_hosts(&args.hosts);
  let unit = 60 * args.interval;
  let mut num = 1;
  // Get logs periodically
  loop {
    println!("\n*** Monitoring at {} minutes ****\n", num * unit / 60);

    time::sleep(time::Duration::from_secs(unit)).await;

    let mut table = Table::new();
    table.add_row(row![
      "current",
      "hostname",
      "status",
      "client_log",
      "init_log"
    ]);

    for host in &hosts {
      let hostname = host.hostname().to_string();
      let ipaddr = host.ipaddr().to_string();
      let status_url = format!(
        "http://{}:{}/status?secret_key=biopoem-N8kOaPq6",
        ipaddr, 3000
      );

      let status = match reqwest::get(status_url).await {
        Err(msg) => "Connection Failed".to_string(),
        Ok(response) => response.text().await.unwrap_or("Running".to_string()),
      };

      let client_log_url = format!(
        "http://{}:{}/log/client?secret_key=biopoem-N8kOaPq6",
        ipaddr, 3000
      );

      let init_log_url = format!(
        "http://{}:{}/log/init?secret_key=biopoem-N8kOaPq6",
        ipaddr, 3000
      );

      let now = chrono::Local::now().format("%Y-%m-%d][%H:%M:%S");
      table.add_row(row![now, hostname, status, client_log_url, init_log_url]);
    }

    table.printstd();
    num += 1;

    // Run query once.
    if !args.online {
      break;
    }
  }
}
