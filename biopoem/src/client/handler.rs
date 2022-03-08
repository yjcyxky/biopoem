use poem::{handler, web::Query, Result};
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Params {
  secret_key: String,
}

fn check_secret_key(secret_key: String) -> bool {
  return secret_key == "biopoem-N8kOaPq6".to_string();
}

fn get_secret_key(res: Result<Query<Params>>) -> String {
  return match res {
    Ok(Query(params)) => params.secret_key,
    Err(err) => "Not valid query".to_string(),
  };
}

#[handler]
pub async fn status(res: Result<Query<Params>>) -> String {
  let secret_key = get_secret_key(res);
  if check_secret_key(secret_key) {
    let status = match fs::read_to_string("status") {
      Err(msg) => "Running".to_string(),
      Ok(msg) => msg,
    };

    return status;
  } else {
    return "Authentication Failed.".to_string();
  }
}

#[handler]
pub async fn client_log(res: Result<Query<Params>>) -> String {
  let secret_key = get_secret_key(res);
  if check_secret_key(secret_key) {
    let task_log = match fs::read_to_string("client.log") {
      Err(msg) => msg.to_string(),
      Ok(msg) => msg,
    };

    return task_log;
  } else {
    return "Authentication Failed.".to_string();
  }
}

#[handler]
pub async fn init_log(res: Result<Query<Params>>) -> String {
  let secret_key = get_secret_key(res);
  if check_secret_key(secret_key) {
    let init_log = match fs::read_to_string("init.log") {
      Err(msg) => msg.to_string(),
      Ok(msg) => msg,
    };

    return init_log;
  } else {
    return "Authentication Failed.".to_string();
  }
}
