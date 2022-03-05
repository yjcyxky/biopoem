use poem::{handler, web::Json};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
pub struct DAGStatus {
  status: String,
  msg: String,
}

#[handler]
pub async fn status() -> Json<DAGStatus> {
  let msg = match fs::read_to_string("client.log") {
    Err(msg) => msg.to_string(),
    Ok(msg) => msg,
  };
  let status = match fs::read_to_string("status") {
    Err(msg) => msg.to_string(),
    Ok(msg) => msg,
  };

  Json(DAGStatus { status, msg })
}
