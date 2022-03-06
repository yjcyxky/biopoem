use serde::{Deserialize, Serialize};
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
pub struct Host {
  hostname: String,
  ipaddr: String,
  port: String,
  username: String,
}

impl Host {
  pub fn hostname(&self) -> &str {
    &self.hostname
  }

  pub fn ipaddr(&self) -> &str {
    &self.ipaddr
  }

  pub fn port(&self) -> &str {
    &self.port
  }

  pub fn username(&self) -> &str {
    &self.username
  }
}

pub fn read_hosts(filepath: &str) -> Vec<Host> {
  let file = File::open(filepath).unwrap();
  let mut hosts: Vec<Host> = vec![];
  let mut rdr = csv::Reader::from_reader(file);
  for result in rdr.deserialize() {
    // Notice that we need to provide a type hint for automatic
    // deserialization.
    let record: Host = result.unwrap();
    hosts.push(record);
  }

  hosts
}
