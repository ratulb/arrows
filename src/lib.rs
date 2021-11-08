#![deny(rust_2018_idioms)]
pub use common::actor::{Actor, ActorBuilder, BuilderResurrector};
pub use common::addr::Addr;
pub use common::errs::{Error, Result};
pub use common::msg::Msg;
pub use common::utils::*;
pub use registry::registry::register;

pub mod common;
pub mod registry;

/***mod actor;
mod addr;
mod errs;
mod msg;
mod utils;***/
