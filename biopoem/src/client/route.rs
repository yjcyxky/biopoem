use crate::client::handler;
use poem::{get, Route};

pub fn init_route() -> Route {
  Route::new()
    .at("status", get(handler::status))
    .at("log/client", get(handler::client_log))
    .at("log/init", get(handler::init_log))
}
