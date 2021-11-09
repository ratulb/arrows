#![deny(rust_2018_idioms)]
pub use common::actor::{Actor, ActorBuilder, BuilderResurrector};
pub use common::addr::Addr;
pub use common::errs::{Error, Result};
pub use common::msg::Msg;
pub use common::utils::*;

pub mod common;
pub mod macros;
pub mod registry;
mod remoting;
