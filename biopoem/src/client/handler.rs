use poem::{handler, web::Path};
use std::fs;

fn check_secret_key(secret_key: String) -> bool {
  return secret_key == "biopoem-N8kOaPq6".to_string();
}

#[handler]
pub async fn status(Path(secret_key): Path<String>) -> String {
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
pub async fn client_log(Path(secret_key): Path<String>) -> String {
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
pub async fn init_log(Path(secret_key): Path<String>) -> String {
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
