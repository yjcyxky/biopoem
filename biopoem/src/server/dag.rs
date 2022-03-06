use log::error;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use tera::{Context, Tera};

pub fn read_to_value(jsonfile: &PathBuf) -> Value {
  let data = fs::read_to_string(jsonfile).unwrap();
  serde_json::from_str(&data).unwrap()
}

pub fn convert_to_ctx(v: Value) -> Context {
  Context::from_value(v).unwrap()
}

pub fn render_template(template: &str, jsonfile: &PathBuf, hostname: &str) -> Option<String> {
  let v = read_to_value(jsonfile);
  let all = convert_to_ctx(v);

  return match all.get(hostname) {
    Some(host_context) => {
      let context = convert_to_ctx(host_context.clone());
      Some(Tera::one_off(template, &context, false).unwrap())
    }
    None => {
      None
    }
  };
}
