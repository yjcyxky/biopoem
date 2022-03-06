use serde_json::Value;
use std::fs;
use tera::{Context, Tera};
use std::path::PathBuf;

pub fn read_to_value(jsonfile: &PathBuf) -> Value {
  let data = fs::read_to_string(jsonfile).unwrap();
  serde_json::from_str(&data).unwrap()
}

pub fn convert_to_ctx(v: Value) -> Context {
  Context::from_value(v).unwrap()
}

pub fn render_template(template: &str, jsonfile: &PathBuf) -> String {
  let v = read_to_value(jsonfile);
  let context = convert_to_ctx(v);

  Tera::one_off(template, &context, false).unwrap()
}
