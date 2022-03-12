use std::{fs, process};
use std::path::Path;
use std::path::PathBuf;
use log::LevelFilter;
use log4rs;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use std::error::Error;

pub mod client;
pub mod server;
pub mod deployer;
pub mod query;

fn notexists_exit(path: &PathBuf, msg: &str) {
  if !Path::exists(path.as_path()) {
    error!("{}", msg);
    process::exit(biopoem_api::PROC_OTHER_ERROR);
  }
}

fn exists_exit(path: &PathBuf, msg: &str) {
  if Path::exists(path.as_path()) {
    error!("{}", msg);
    process::exit(biopoem_api::PROC_OTHER_ERROR);
  }
}

fn init_logger(tag_name: &str) -> Result<log4rs::Handle, String> {
  let stdout = ConsoleAppender::builder()
    .encoder(Box::new(PatternEncoder::new(
      &(format!("[{}]", tag_name) + " {d} - {l} -{t} - {m}{n}"),
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

fn init_file_logger(tag_name: &str, logpath: &str) -> Result<log4rs::Handle, String> {
  let stdout = ConsoleAppender::builder()
    .encoder(Box::new(PatternEncoder::new(
      &(format!("[{}]", tag_name) + " {d} - {l} -{t} - {m}{n}"),
    )))
    .build();

  match fs::remove_file(logpath) {
    _ => {}
  };

  let file = FileAppender::builder()
    .encoder(Box::new(PatternEncoder::new(
      "[File] {d} - {l} - {t} - {m}{n}",
    )))
    .build(logpath)
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