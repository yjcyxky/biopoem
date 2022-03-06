#[macro_use]
extern crate lazy_static;

pub mod client;
pub mod server;

pub const PROC_SUCCESS: i32 = 0;
pub const PROC_PARSE_ERROR: i32 = 1;
pub const PROC_EXEC_ERROR: i32 = 2;
pub const PROC_OTHER_ERROR: i32 = 3;