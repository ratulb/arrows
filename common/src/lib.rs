#![deny(rust_2018_idioms)]

pub use actor::{Actor, ActorBuilder, BuilderResurrector};
pub use addr::Addr;
pub use errs::{Error, Result};
pub use msg::Msg;
pub use utils::*;

mod actor;
mod addr;
mod errs;
mod msg;
mod utils;
