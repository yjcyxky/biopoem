use poem::{handler, web::Json};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
pub struct DAGStatus {
  status: String,
  task_log: String,
  init_log: String,
}

#[handler]
pub async fn status() -> Json<DAGStatus> {
  let task_log = match fs::read_to_string("client.log") {
    Err(msg) => msg.to_string(),
    Ok(msg) => msg,
  };

  let status = match fs::read_to_string("status") {
    Err(msg) => msg.to_string(),
    Ok(msg) => msg,
  };

  let init_log = match fs::read_to_string("init.log") {
    Err(msg) => msg.to_string(),
    Ok(msg) => msg,
  };

  Json(DAGStatus {
    status,
    init_log,
    task_log,
  })
}
