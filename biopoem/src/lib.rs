#[macro_use]
extern crate lazy_static;

use log::{error, info};
use std::fs;
use std::path::Path;
use std::process::{self, Output};
use std::str;

pub mod client;
pub mod deployer;
pub mod server;

pub const PROC_SUCCESS: i32 = 0;
pub const PROC_PARSE_ERROR: i32 = 1;
pub const PROC_EXEC_ERROR: i32 = 2;
pub const PROC_OTHER_ERROR: i32 = 3;

pub fn makedir(dir: &str) {
  if !Path::new(&dir).exists() {
    match fs::create_dir_all(dir) {
      Err(err) => {
        println!("Cannot create the directory {}, {}", dir, err);
        process::exit(PROC_OTHER_ERROR);
      }
      Ok(_) => {
        println!("Create the directory {}", dir);
      }
    };
  }
}

#[derive(Debug, PartialEq)]
pub enum Status {
  Success,
  Failed,
}

pub fn vecu8_to_string(data: &Vec<u8>) -> String {
  let s = match str::from_utf8(data) {
    Ok(v) => v.to_string(),
    Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
  };

  return s;
}

pub fn handle_output(output: &Output) -> Status {
  info!("Output:\n{}", vecu8_to_string(&output.stdout));

  if output.stderr.len() > 0 {
    error!("Error:\n{}", vecu8_to_string(&output.stderr));
  }

  if output.status.success() {
    return Status::Success;
  } else {
    return Status::Failed;
  }
}
