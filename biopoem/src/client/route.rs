use crate::client::handler;
use poem::{get, Route};

pub fn init_route() -> Route {
  Route::new()
    .at("status", get(handler::status))
}
