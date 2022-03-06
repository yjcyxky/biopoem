use log::{error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::str;
use tera::{Context, Tera};

#[derive(Debug, Deserialize, Serialize)]
pub struct Server {
  host_name: String,
  instance_name: String,
  instance_type: String,
  key_name: String,
  private_ip: String,
  public_ip: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
  region: String,
  zone: String,
  ipaddrs: Vec<String>,
  num_of_hosts: usize,
  image: String,
  instance_type: String,
  keypair_name: String,
}

impl Config {
  pub fn new(
    region: &str,
    zone: &str,
    num_of_hosts: usize,
    image: &str,
    instance_type: &str,
    keypair_name: &str,
  ) -> Self {
    let mut ipaddrs: Vec<String> = vec![];

    if num_of_hosts > 256 {
      error!("You cannot deploy more than 255 servers.");
      std::process::exit(127);
    }

    for i in 1..num_of_hosts {
      ipaddrs.push(format!("172.16.0.{}", i));
    }

    Config {
      region: region.to_string(),
      zone: format!("{}-{}", region, zone),
      ipaddrs: ipaddrs,
      num_of_hosts: num_of_hosts,
      image: image.to_string(),
      instance_type: instance_type.to_string(),
      keypair_name: keypair_name.to_string(),
    }
  }
}

pub fn render_template(template: &str, data: &Config) -> Option<String> {
  let context = Context::from_serialize(data).unwrap();
  Some(Tera::one_off(template, &context, false).unwrap())
}

fn vecu8_to_string(data: &Vec<u8>) -> String {
  let s = match str::from_utf8(data) {
    Ok(v) => v.to_string(),
    Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
  };

  return s;
}

pub fn run(command: &str, access_key: &str, secret_key: &str, region: &str) -> Option<String> {
  let mut commands = HashMap::new();
  commands.insert("init", vec!["init", "-input=false"]);
  commands.insert("apply", vec!["apply", "-input=false"]);
  commands.insert("show", vec!["show", "-json"]);
  commands.insert("destroy", vec!["destroy", "-input=true"]);

  let args = commands.get(command).unwrap();

  match Command::new("terraform")
    .env("ALICLOUD_ACCESS_KEY", access_key)
    .env("ALICLOUD_SECRET_KEY", secret_key)
    .env("ALICLOUD_REGION", region)
    .args(args)
    .output()
  {
    Err(msg) => {
      error!("Cannot run terraform with {:?}, {}", args, msg);
      return None;
    }
    Ok(output) => {
      info!("{:?}", output);
      return Some(vecu8_to_string(&output.stdout));
    }
  };
}
