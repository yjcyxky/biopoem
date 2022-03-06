#[macro_use]
extern crate lazy_static;

use std::fs;
use std::path::Path;
use std::process;

pub mod client;
pub mod server;
pub mod deployer;

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
