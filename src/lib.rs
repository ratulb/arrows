//#![deny(unsafe_code)]
#![deny(rust_2018_idioms)]
pub use common::actor::{Actor, ActorBuilder, BuilderDeserializer};
pub use common::addr::Addr;
pub use common::errs::{Error, Result};
pub use common::mail::Mail;
pub use common::mail::Msg;
pub use common::utils::*;

pub mod common;
pub mod macros;
pub mod registry;
mod remoting;
mod routers;
mod sender;
