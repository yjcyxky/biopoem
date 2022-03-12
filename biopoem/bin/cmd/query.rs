use super::init_logger;
use super::notexists_exit;
use biopoem_api::server;
use chrono;
use prettytable::Table;
use reqwest;
use std::path::PathBuf;
use std::process;
use structopt::StructOpt;
use tokio::{self, time};

/// Query task status for Biopoem
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

#[tokio::main]
pub async fn run(args: &Arguments) {
  if let Err(log) = init_logger("Query") {
    error!(target:"stdout", "Log initialization error, {}", log);
    process::exit(biopoem_api::PROC_OTHER_ERROR);
  };

  notexists_exit(
    &PathBuf::from(&args.hosts),
    &format!("No such file: {} file doesn't exist.", &args.hosts),
  );

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
