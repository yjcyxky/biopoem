use log::{error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{Command, Output};
use std::str;
use tera::{Context, Tera};

#[derive(Debug, Deserialize, Serialize)]
pub struct Host {
  hostname: String,
  ipaddr: String,
  private_ipaddr: String,
  port: String,
  username: String,
}

impl Host {
  pub fn new(
    hostname: String,
    ipaddr: String,
    private_ipaddr: String,
    port: String,
    username: String,
  ) -> Self {
    Host {
      hostname: hostname,
      ipaddr: ipaddr,
      private_ipaddr: private_ipaddr,
      port: port,
      username: username,
    }
  }
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

    for i in 0..num_of_hosts {
      ipaddrs.push(format!("172.16.0.{}", i + 1));
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

pub fn gen_hosts(data: &Config, public_ips: &Vec<String>) -> Vec<Host> {
  let mut hosts: Vec<Host> = vec![];
  for (idx, ipaddr) in data.ipaddrs.iter().enumerate() {
    hosts.push(Host {
      hostname: format!("biopoem{:03}", idx + 1),
      private_ipaddr: ipaddr.clone(),
      ipaddr: public_ips[idx].to_string(),
      port: "22".to_string(),
      username: "root".to_string(),
    })
  }

  hosts
}

pub fn render_template(template: &str, data: &Config) -> Option<String> {
  let context = Context::from_serialize(data).unwrap();
  Some(Tera::one_off(template, &context, false).unwrap())
}

pub fn run(
  command: &str,
  dir: &str,
  access_key: &str,
  secret_key: &str,
  region: &str,
) -> Option<Output> {
  let mut commands = HashMap::new();
  commands.insert("init", vec!["init", "-input=false"]);
  commands.insert("apply", vec!["apply", "-auto-approve", "-input=false"]);
  commands.insert("show", vec!["show", "-json"]);
  commands.insert("destroy", vec!["destroy", "-auto-approve", "-input=false"]);
  commands.insert("output", vec!["output", "-json", "public_ips"]);

  let args = commands.get(command).unwrap();

  match Command::new("terraform")
    .env("ALICLOUD_ACCESS_KEY", access_key)
    .env("ALICLOUD_SECRET_KEY", secret_key)
    .env("ALICLOUD_REGION", region)
    .current_dir(dir)
    .args(args)
    .output()
  {
    Err(msg) => {
      error!("Cannot run terraform with {:?}, {}", args, msg);
      return None;
    }
    Ok(output) => {
      return Some(output);
    }
  };
}
